use anyhow::Result;
use eframe::egui;
use howler_core::Database;
use lru::LruCache;
use std::num::NonZeroUsize;
use std::sync::Arc;
use tokio::sync::Mutex;

pub struct HowlerApp {
    sightings: Vec<SightingData>,
    selected_sighting: Option<usize>,
    zoom: f32,
    pan: egui::Vec2,
    tile_cache: Arc<Mutex<LruCache<TileKey, TileData>>>,
    rt: tokio::runtime::Runtime,
    show_gbif: bool,
    show_movebank: bool,
    show_inaturalist: bool,
    show_pack_territories: bool,
}

#[derive(Clone, PartialEq, Eq, Hash)]
struct TileKey {
    x: u32,
    y: u32,
    z: u8,
}

struct TileData {
    texture: egui::TextureHandle,
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

        let rt = tokio::runtime::Runtime::new()?;
        let tile_cache = Arc::new(Mutex::new(LruCache::new(NonZeroUsize::new(100).unwrap())));

        Ok(Self {
            sightings: sighting_data,
            selected_sighting: None,
            zoom: 1.0,
            pan: egui::Vec2::ZERO,
            tile_cache,
            rt,
            show_gbif: true,
            show_movebank: true,
            show_inaturalist: true,
            show_pack_territories: false,
        })
    }

    #[allow(dead_code)]
    fn lat_lon_to_tile(&self, lat: f64, lon: f64, zoom: u8) -> (u32, u32) {
        let n = 1 << zoom;
        let x = ((lon + 180.0) / 360.0 * n as f64).floor() as u32;
        let lat_rad = lat.to_radians();
        let y = ((1.0 - lat_rad.tan().ln() + std::f64::consts::PI / 2.0) / std::f64::consts::PI
            * n as f64)
            .floor() as u32;
        (x, y)
    }

    #[allow(dead_code)]
    fn tile_to_lat_lon(&self, x: u32, y: u32, zoom: u8) -> (f64, f64) {
        let n = 1 << zoom;
        let lon = x as f64 / n as f64 * 360.0 - 180.0;
        let lat_rad = (std::f64::consts::PI * (1.0 - 2.0 * y as f64 / n as f64))
            .exp()
            .atan();
        let lat = lat_rad.to_degrees();
        (lat, lon)
    }

    #[allow(dead_code)]
    async fn fetch_tile(&self, x: u32, y: u32, z: u8) -> Result<Vec<u8>> {
        let url = format!("https://tile.openstreetmap.org/{}/{}/{}.png", z, x, y);

        let response = reqwest::get(&url).await?;
        let bytes = response.bytes().await?;
        Ok(bytes.to_vec())
    }

    fn get_or_fetch_tile(
        &self,
        ctx: &egui::Context,
        x: u32,
        y: u32,
        z: u8,
    ) -> Option<egui::TextureHandle> {
        let key = TileKey { x, y, z };

        // Check cache
        {
            let mut cache = self.tile_cache.blocking_lock();
            if let Some(tile_data) = cache.get(&key) {
                return Some(tile_data.texture.clone());
            }
        }

        // Fetch tile asynchronously
        let cache_clone = self.tile_cache.clone();
        let key_clone = key.clone();
        let ctx_clone = ctx.clone();

        self.rt.spawn(async move {
            let bytes = match reqwest::get(&format!(
                "https://tile.openstreetmap.org/{}/{}/{}.png",
                z, x, y
            ))
            .await
            {
                Ok(resp) => resp.bytes().await,
                Err(e) => {
                    eprintln!("Failed to fetch tile: {}", e);
                    return;
                }
            };

            let bytes = match bytes {
                Ok(b) => b,
                Err(e) => {
                    eprintln!("Failed to read tile bytes: {}", e);
                    return;
                }
            };

            let image = match image::load_from_memory(&bytes) {
                Ok(img) => img.to_rgba8(),
                Err(e) => {
                    eprintln!("Failed to decode tile: {}", e);
                    return;
                }
            };

            let size = [image.width() as usize, image.height() as usize];
            let pixels = image.into_raw();

            ctx_clone.request_repaint();

            let mut cache = cache_clone.lock().await;
            let texture = ctx_clone.load_texture(
                "osm_tile",
                egui::ColorImage::from_rgba_unmultiplied(size, &pixels),
                egui::TextureOptions::LINEAR,
            );

            cache.put(key_clone, TileData { texture });
        });

        None
    }

    fn lat_lon_to_screen(&self, lat: f64, lon: f64, rect: egui::Rect) -> egui::Pos2 {
        // Simple equirectangular projection
        let x = ((lon + 180.0) / 360.0) * rect.width() as f64;
        let y = ((90.0 - lat) / 180.0) * rect.height() as f64;

        egui::Pos2::new(x as f32, y as f32)
    }
}

impl eframe::App for HowlerApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
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

            ui.separator();

            // Map canvas
            let desired_size = egui::vec2(ui.available_width(), 400.0);
            let (map_rect, map_response) =
                ui.allocate_exact_size(desired_size, egui::Sense::click_and_drag());

            let painter = ui.painter_at(map_rect);

            // Draw OSM tiles
            let zoom_level = (self.zoom * 5.0).clamp(0.0, 18.0) as u8;
            let tile_size = 256.0;

            // Calculate visible tile range
            let center_x = map_rect.center().x - self.pan.x;
            let center_y = map_rect.center().y - self.pan.y;

            let start_tile_x = ((center_x / tile_size - 2.0).floor() as i32).max(0) as u32;
            let start_tile_y = ((center_y / tile_size - 2.0).floor() as i32).max(0) as u32;
            let end_tile_x = start_tile_x + 5;
            let end_tile_y = start_tile_y + 5;

            for tile_x in start_tile_x..=end_tile_x {
                for tile_y in start_tile_y..=end_tile_y {
                    let screen_x = map_rect.min.x + tile_x as f32 * tile_size + self.pan.x;
                    let screen_y = map_rect.min.y + tile_y as f32 * tile_size + self.pan.y;

                    let tile_rect = egui::Rect::from_min_size(
                        egui::Pos2::new(screen_x, screen_y),
                        egui::vec2(tile_size, tile_size),
                    );

                    if let Some(texture) = self.get_or_fetch_tile(ctx, tile_x, tile_y, zoom_level) {
                        painter.image(
                            texture.id(),
                            tile_rect,
                            egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
                            egui::Color32::WHITE,
                        );
                    } else {
                        // Draw placeholder
                        painter.rect_filled(tile_rect, 0.0, egui::Color32::from_rgb(200, 200, 200));
                    }
                }
            }

            // Handle pan
            if map_response.dragged() {
                self.pan += map_response.drag_delta();
            }

            // Handle zoom
            if map_response.hovered() {
                let scroll = ui.input(|i| i.raw_scroll_delta);
                if scroll.y != 0.0 {
                    self.zoom = (self.zoom * (1.0 + scroll.y * 0.001)).clamp(0.2, 10.0);
                }
            }

            // Draw pack territories if enabled
            if self.show_pack_territories {
                // Simple placeholder for pack territory visualization
                // In a full implementation, this would draw convex hull polygons
                painter.rect_stroke(
                    egui::Rect::from_min_size(
                        egui::Pos2::new(
                            map_rect.min.x + 100.0 + self.pan.x,
                            map_rect.min.y + 100.0 + self.pan.y,
                        ),
                        egui::vec2(200.0 * self.zoom, 150.0 * self.zoom),
                    ),
                    0.0,
                    (2.0, egui::Color32::from_rgb(255, 100, 100)),
                );
            }

            // Draw sightings with layer filtering
            for (i, sighting) in self.sightings.iter().enumerate() {
                // Check if this source layer is visible
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
                let pos = pos + self.pan * self.zoom;

                // Color by source
                let color = match sighting.source.as_str() {
                    "GBIF" => egui::Color32::LIGHT_BLUE,
                    "Movebank" => egui::Color32::LIGHT_GREEN,
                    "iNaturalist" => egui::Color32::LIGHT_YELLOW,
                    _ => egui::Color32::LIGHT_GRAY,
                };

                // Highlight selected
                let radius = if self.selected_sighting == Some(i) {
                    8.0 * self.zoom
                } else {
                    5.0 * self.zoom
                };

                painter.circle_filled(pos, radius, color);

                // Handle click selection
                if map_response.clicked() {
                    let click_pos = map_response.hover_pos().unwrap_or(egui::Pos2::ZERO);
                    let dist = click_pos.distance(pos);
                    if dist < radius * 2.0 {
                        self.selected_sighting = Some(i);
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
                }
            } else {
                ui.label("Click on a marker to see details");
            }

            ui.separator();
            ui.label(format!("Total sightings: {}", self.sightings.len()));
            ui.label("Legend: Blue=GBIF, Green=Movebank, Yellow=iNaturalist");
            ui.label("Controls: Drag to pan, scroll to zoom");
            ui.label(format!("Zoom level: {:.1}", self.zoom));
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
