use anyhow::Result;
use eframe::egui;
use howler_core::Database;
use std::collections::HashMap;

pub struct HowlerApp {
    sightings: Vec<SightingData>,
    selected_sighting: Option<usize>,
    zoom: f32,
    pan: egui::Vec2,
    center_lat: f64,
    center_lon: f64,
    show_gbif: bool,
    show_movebank: bool,
    show_inaturalist: bool,
    show_pack_territories: bool,
    fly_to: Option<(f64, f64, f32)>,
    dark_mode: bool,
    touch_points: HashMap<u64, egui::Pos2>,
    last_pinch_dist: Option<f32>,
}

struct SightingData {
    species: String,
    latitude: f64,
    longitude: f64,
    date: String,
    source: String,
    details: Option<String>,
}

impl HowlerApp {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Result<Self> {
        let db = Database::new("howler.db")?;
        let sightings = db.get_all_sightings()?;

        let sighting_data = sightings
            .into_iter()
            .map(|s| SightingData {
                species: s.species,
                latitude: s.latitude,
                longitude: s.longitude,
                date: s.observed_on.format("%Y-%m-%d").to_string(),
                source: s.source.to_string(),
                details: s.details,
            })
            .collect();

        Ok(Self {
            sightings: sighting_data,
            selected_sighting: None,
            zoom: 1.0,
            pan: egui::Vec2::ZERO,
            center_lat: 44.4280,
            center_lon: -110.5885,
            show_gbif: true,
            show_movebank: true,
            show_inaturalist: true,
            show_pack_territories: false,
            fly_to: None,
            dark_mode: false,
            touch_points: HashMap::new(),
            last_pinch_dist: None,
        })
    }

    fn degrees_per_pixel(&self, map_width: f32) -> f64 {
        360.0 / (map_width as f64 * self.zoom as f64)
    }

    fn lat_lon_to_screen(&self, lat: f64, lon: f64, map_rect: egui::Rect) -> egui::Pos2 {
        let dpp = self.degrees_per_pixel(map_rect.width());
        let cx = map_rect.center().x + self.pan.x;
        let cy = map_rect.center().y + self.pan.y;
        let x = cx + ((lon - self.center_lon) / dpp) as f32;
        let y = cy - ((lat - self.center_lat) / dpp) as f32;
        egui::Pos2::new(x, y)
    }

    fn screen_to_lat_lon(&self, pos: egui::Pos2, map_rect: egui::Rect) -> (f64, f64) {
        let dpp = self.degrees_per_pixel(map_rect.width());
        let cx = map_rect.center().x + self.pan.x;
        let cy = map_rect.center().y + self.pan.y;
        let lon = self.center_lon + (pos.x - cx) as f64 * dpp;
        let lat = self.center_lat - (pos.y - cy) as f64 * dpp;
        (lat, lon)
    }

    fn grid_interval(&self) -> f64 {
        let dpp = self.degrees_per_pixel(800.0);
        let targets = [
            0.01, 0.02, 0.05, 0.1, 0.2, 0.5, 1.0, 2.0, 5.0, 10.0, 15.0, 20.0, 30.0, 45.0,
        ];
        let pixel_target = 120.0;
        for &t in &targets {
            if t / dpp >= pixel_target {
                return t;
            }
        }
        45.0
    }
}

impl eframe::App for HowlerApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Detect OS dark mode once
        if !self.dark_mode && ctx.style().visuals.dark_mode {
            self.dark_mode = true;
        }

        // Apply theme if changed
        let current_is_dark = ctx.style().visuals.dark_mode;
        if self.dark_mode != current_is_dark {
            if self.dark_mode {
                ctx.set_visuals(egui::Visuals::dark());
            } else {
                ctx.set_visuals(egui::Visuals::light());
            }
        }

        // Fly-to animation
        if let Some((target_lat, target_lon, target_zoom)) = self.fly_to {
            let lerp_factor = 0.15_f64;
            self.center_lat += (target_lat - self.center_lat) * lerp_factor;
            self.center_lon += (target_lon - self.center_lon) * lerp_factor;
            self.zoom += (target_zoom - self.zoom) * lerp_factor as f32;
            self.pan = egui::Vec2::ZERO;

            let lat_done = (self.center_lat - target_lat).abs() < 0.001;
            let lon_done = (self.center_lon - target_lon).abs() < 0.001;
            let zoom_done = (self.zoom - target_zoom).abs() < 0.05;
            if lat_done && lon_done && zoom_done {
                self.center_lat = target_lat;
                self.center_lon = target_lon;
                self.zoom = target_zoom;
                self.fly_to = None;
            }
            ctx.request_repaint();
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Howler - Wolf Tracking Map");

            // Layer controls
            ui.horizontal(|ui| {
                ui.label("Layers:");
                ui.checkbox(&mut self.show_gbif, "GBIF");
                ui.checkbox(&mut self.show_movebank, "Movebank");
                ui.checkbox(&mut self.show_inaturalist, "iNaturalist");
                ui.checkbox(&mut self.show_pack_territories, "Pack Territories");
            });

            ui.horizontal(|ui| {
                ui.checkbox(&mut self.dark_mode, "Dark Mode");
            });

            ui.separator();

            // Map canvas
            let desired_size = egui::vec2(ui.available_width(), 400.0);
            let (map_rect, map_response) =
                ui.allocate_exact_size(desired_size, egui::Sense::click_and_drag());

            let painter = ui.painter_at(map_rect);

            // Background
            let bg = if self.dark_mode {
                egui::Color32::from_rgb(30, 30, 30)
            } else {
                egui::Color32::from_rgb(245, 245, 245)
            };
            painter.rect_filled(map_rect, 0.0, bg);

            // Grid lines
            let interval = self.grid_interval();
            let grid_color = if self.dark_mode {
                egui::Color32::from_rgb(60, 60, 60)
            } else {
                egui::Color32::from_rgb(200, 200, 200)
            };
            let label_color = if self.dark_mode {
                egui::Color32::from_rgb(140, 140, 140)
            } else {
                egui::Color32::from_rgb(120, 120, 120)
            };

            // Visible lat/lon bounds
            let (top_lat, _left_lon) =
                self.screen_to_lat_lon(egui::pos2(map_rect.left(), map_rect.top()), map_rect);
            let (bottom_lat, _right_lon) =
                self.screen_to_lat_lon(egui::pos2(map_rect.right(), map_rect.bottom()), map_rect);
            let (_top_lat, left_lon) =
                self.screen_to_lat_lon(egui::pos2(map_rect.left(), map_rect.top()), map_rect);
            let (_bottom_lat, right_lon) =
                self.screen_to_lat_lon(egui::pos2(map_rect.right(), map_rect.bottom()), map_rect);

            let lat_min = bottom_lat.min(top_lat);
            let lat_max = bottom_lat.max(top_lat);
            let lon_min = left_lon.min(right_lon);
            let lon_max = left_lon.max(right_lon);

            // Horizontal grid lines (latitude)
            let first_lat = (lat_min / interval).floor() * interval;
            let mut lat = first_lat;
            while lat <= lat_max {
                let pos = self.lat_lon_to_screen(lat, self.center_lon, map_rect);
                if map_rect.y_range().contains(pos.y) {
                    painter.line_segment(
                        [
                            egui::pos2(map_rect.left(), pos.y),
                            egui::pos2(map_rect.right(), pos.y),
                        ],
                        egui::Stroke::new(1.0_f32, grid_color),
                    );
                    painter.text(
                        egui::pos2(map_rect.left() + 4.0, pos.y + 2.0),
                        egui::Align2::LEFT_TOP,
                        format!("{:.1}°", lat),
                        egui::FontId::proportional(10.0),
                        label_color,
                    );
                }
                lat += interval;
            }

            // Vertical grid lines (longitude)
            let first_lon = (lon_min / interval).floor() * interval;
            let mut lon = first_lon;
            while lon <= lon_max {
                let pos = self.lat_lon_to_screen(self.center_lat, lon, map_rect);
                if map_rect.x_range().contains(pos.x) {
                    painter.line_segment(
                        [
                            egui::pos2(pos.x, map_rect.top()),
                            egui::pos2(pos.x, map_rect.bottom()),
                        ],
                        egui::Stroke::new(1.0_f32, grid_color),
                    );
                    painter.text(
                        egui::pos2(pos.x + 2.0, map_rect.top() + 2.0),
                        egui::Align2::LEFT_TOP,
                        format!("{:.1}°", lon),
                        egui::FontId::proportional(10.0),
                        label_color,
                    );
                }
                lon += interval;
            }

            // Handle pan
            if map_response.dragged() {
                self.pan += map_response.drag_delta();
            }

            // Handle zoom (toward cursor position)
            if map_response.hovered() {
                let scroll = ui.input(|i| i.raw_scroll_delta);
                if scroll.y != 0.0 {
                    let old_zoom = self.zoom;
                    self.zoom = (self.zoom * (1.0 + scroll.y * 0.001)).clamp(0.2, 10.0);

                    // Zoom toward cursor
                    if let Some(cursor) = map_response.hover_pos() {
                        let zoom_ratio = self.zoom / old_zoom;
                        let offset_from_center = cursor - map_rect.center();
                        let new_offset = offset_from_center * zoom_ratio;
                        self.pan =
                            (cursor - map_rect.center()) - new_offset + self.pan * zoom_ratio;
                    }
                }
            }

            // Multi-touch pinch-to-zoom
            ui.input(|i| {
                for touch in &i.raw.events {
                    if let egui::Event::Touch { id, phase, pos, .. } = touch {
                        match phase {
                            egui::TouchPhase::Start => {
                                self.touch_points.insert(id.0, *pos);
                            }
                            egui::TouchPhase::Move => {
                                self.touch_points.insert(id.0, *pos);
                            }
                            egui::TouchPhase::End | egui::TouchPhase::Cancel => {
                                self.touch_points.remove(&id.0);
                            }
                        }
                    }
                }

                if self.touch_points.len() == 2 {
                    let points: Vec<egui::Pos2> = self.touch_points.values().copied().collect();
                    let dist = points[0].distance(points[1]);

                    if let Some(prev_dist) = self.last_pinch_dist {
                        if prev_dist > 1.0 {
                            let scale = dist / prev_dist;
                            let old_zoom = self.zoom;
                            self.zoom = (self.zoom * scale).clamp(0.2, 10.0);

                            let midpoint = (points[0].to_vec2() + points[1].to_vec2()) * 0.5;
                            let midpoint = midpoint.to_pos2();
                            let zoom_ratio = self.zoom / old_zoom;
                            let offset_from_center = midpoint - map_rect.center();
                            let new_offset = offset_from_center * zoom_ratio;
                            self.pan =
                                (midpoint - map_rect.center()) - new_offset + self.pan * zoom_ratio;
                        }
                    }
                    self.last_pinch_dist = Some(dist);
                } else {
                    self.last_pinch_dist = None;
                }
            });

            // Draw pack territories if enabled
            if self.show_pack_territories {
                let tl = self.lat_lon_to_screen(44.55, -110.6, map_rect);
                let br = self.lat_lon_to_screen(44.45, -110.4, map_rect);
                let rect = egui::Rect::from_min_max(tl, br);
                painter.rect_stroke(rect, 0.0, (2.0, egui::Color32::from_rgb(255, 100, 100)));
            }

            // Draw sightings with layer filtering
            for (i, sighting) in self.sightings.iter().enumerate() {
                let visible = match sighting.source.as_str() {
                    "GBIF" => self.show_gbif,
                    "Movebank" => self.show_movebank,
                    "iNaturalist" => self.show_inaturalist,
                    _ => true,
                };

                if !visible {
                    continue;
                }

                let pos = self.lat_lon_to_screen(sighting.latitude, sighting.longitude, map_rect);

                if !map_rect.contains(pos) {
                    continue;
                }

                let color = match sighting.source.as_str() {
                    "GBIF" => egui::Color32::LIGHT_BLUE,
                    "Movebank" => egui::Color32::LIGHT_GREEN,
                    "iNaturalist" => egui::Color32::LIGHT_YELLOW,
                    _ => egui::Color32::LIGHT_GRAY,
                };

                let radius = if self.selected_sighting == Some(i) {
                    8.0 * self.zoom
                } else {
                    5.0 * self.zoom
                };

                painter.circle_filled(pos, radius, color);

                if map_response.clicked() {
                    let click_pos = map_response.hover_pos().unwrap_or(egui::Pos2::ZERO);
                    let dist = click_pos.distance(pos);
                    if dist < radius * 2.0 {
                        self.selected_sighting = Some(i);
                        self.fly_to = Some((sighting.latitude, sighting.longitude, 6.0));
                    }
                }
            }

            // Details panel
            ui.separator();
            if let Some(idx) = self.selected_sighting {
                if let Some(sighting) = self.sightings.get(idx) {
                    ui.heading("Sighting Details");
                    ui.label(format!("Species: {}", sighting.species));
                    ui.label(format!("Date: {}", sighting.date));
                    ui.label(format!("Source: {}", sighting.source));
                    ui.label(format!("Latitude: {:.4}", sighting.latitude));
                    ui.label(format!("Longitude: {:.4}", sighting.longitude));
                    ui.label(format!(
                        "Details: {}",
                        sighting.details.as_deref().unwrap_or("N/A")
                    ));
                    if ui.button("Zoom to").clicked() {
                        self.fly_to = Some((sighting.latitude, sighting.longitude, 6.0));
                    }
                }
            } else {
                ui.label("Click on a marker to see details");
            }

            ui.separator();
            ui.label(format!("Total sightings: {}", self.sightings.len()));
            ui.label("Legend: Blue=GBIF, Green=Movebank, Yellow=iNaturalist");
            ui.label("Controls: Drag to pan, scroll to zoom");
            ui.label(format!(
                "Zoom: {:.1}x | Center: {:.4}, {:.4}",
                self.zoom, self.center_lat, self.center_lon
            ));
        });
    }
}

pub fn run() -> Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1200.0, 800.0])
            .with_title("Howler - Wolf Tracking"),
        ..Default::default()
    };

    eframe::run_native(
        "Howler",
        options,
        Box::new(|cc| Ok(Box::new(HowlerApp::new(cc).expect("Failed to create app")))),
    )
    .map_err(|e| anyhow::anyhow!("GUI error: {}", e))
}
