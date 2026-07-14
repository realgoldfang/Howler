use anyhow::Result;
use eframe::egui;
use howler_core::Database;
use lru::LruCache;
use std::collections::HashSet;
use std::num::NonZeroUsize;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::{Mutex, Semaphore};

pub struct HowlerApp {
    sightings: Vec<SightingData>,
    selected_sighting: Option<usize>,
    zoom: f32,
    pan: egui::Vec2,
    center_lat: f64,
    center_lon: f64,
    tile_cache: Arc<Mutex<LruCache<TileKey, TileData>>>,
    failed_tiles: Arc<Mutex<HashSet<TileKey>>>,
    fetch_in_progress: Arc<Mutex<HashSet<TileKey>>>,
    semaphore: Arc<Semaphore>,
    rt: tokio::runtime::Runtime,
    show_gbif: bool,
    show_movebank: bool,
    show_inaturalist: bool,
    show_pack_territories: bool,
    offline_mode: bool,
    disk_cache_dir: PathBuf,
    fly_to: Option<(f64, f64, f32)>,
    dark_mode: bool,
}

#[derive(Clone, PartialEq, Eq, Hash)]
struct TileKey {
    x: u32,
    y: u32,
    z: u8,
}

#[derive(Clone)]
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
        let tile_cache = Arc::new(Mutex::new(LruCache::new(NonZeroUsize::new(200).unwrap())));
        let failed_tiles = Arc::new(Mutex::new(HashSet::new()));
        let fetch_in_progress = Arc::new(Mutex::new(HashSet::new()));
        let semaphore = Arc::new(Semaphore::new(2));

        let disk_cache_dir = dirs::data_local_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("howler")
            .join("tile_cache");
        std::fs::create_dir_all(&disk_cache_dir).ok();

        Ok(Self {
            sightings: sighting_data,
            selected_sighting: None,
            zoom: 1.0,
            pan: egui::Vec2::ZERO,
            center_lat: 44.4280,
            center_lon: -110.5885,
            tile_cache,
            failed_tiles,
            fetch_in_progress,
            semaphore,
            rt,
            show_gbif: true,
            show_movebank: true,
            show_inaturalist: true,
            show_pack_territories: false,
            offline_mode: false,
            disk_cache_dir,
            fly_to: None,
            dark_mode: false,
        })
    }

    fn zoom_level(&self) -> u8 {
        (self.zoom * 2.0).clamp(0.0, 18.0) as u8
    }

    fn lat_lon_to_tile(&self, lat: f64, lon: f64, zoom: u8) -> (u32, u32) {
        let n = 1 << zoom;
        let x = ((lon + 180.0) / 360.0 * n as f64).floor() as u32;
        let lat_rad = lat.to_radians();
        let y = ((1.0 - lat_rad.tan().ln() + std::f64::consts::PI / 2.0) / std::f64::consts::PI
            * n as f64)
            .floor() as u32;
        (x, y)
    }

    fn tile_disk_path(&self, x: u32, y: u32, z: u8) -> PathBuf {
        self.disk_cache_dir
            .join(z.to_string())
            .join(x.to_string())
            .join(format!("{}.png", y))
    }

    fn load_tile_from_disk(&self, ctx: &egui::Context, key: &TileKey) -> Option<TileData> {
        let path = self.tile_disk_path(key.x, key.y, key.z);
        let bytes = std::fs::read(&path).ok()?;
        let img = image::load_from_memory(&bytes).ok()?;
        let rgba = img.to_rgba8();
        let size = [rgba.width() as usize, rgba.height() as usize];
        let pixels = rgba.into_raw();
        let texture = ctx.load_texture(
            format!("tile_{}_{}_{}", key.z, key.x, key.y),
            egui::ColorImage::from_rgba_unmultiplied(size, &pixels),
            egui::TextureOptions::LINEAR,
        );
        Some(TileData { texture })
    }

    fn get_or_fetch_tile(
        &self,
        ctx: &egui::Context,
        x: u32,
        y: u32,
        z: u8,
    ) -> Option<egui::TextureHandle> {
        let key = TileKey { x, y, z };

        // Check in-memory cache
        {
            let mut cache = self.tile_cache.blocking_lock();
            if let Some(tile_data) = cache.get(&key) {
                return Some(tile_data.texture.clone());
            }
        }

        // Don't re-fetch if already in progress or recently failed
        {
            let in_progress = self.fetch_in_progress.blocking_lock();
            if in_progress.contains(&key) {
                return None;
            }
        }

        // Try disk cache
        if let Some(disk_tile) = self.load_tile_from_disk(ctx, &key) {
            let mut cache = self.tile_cache.blocking_lock();
            cache.put(key.clone(), disk_tile.clone());
            return Some(disk_tile.texture);
        }

        // In offline mode, don't try network
        if self.offline_mode {
            return None;
        }

        // Spawn network fetch
        let cache_clone = self.tile_cache.clone();
        let failed_clone = self.failed_tiles.clone();
        let in_progress_clone = self.fetch_in_progress.clone();
        let sem_clone = self.semaphore.clone();
        let ctx_clone = ctx.clone();
        let key_clone = key.clone();
        let disk_dir = self.disk_cache_dir.clone();

        self.rt.spawn(async move {
            // Mark as in progress
            {
                let mut ip = in_progress_clone.lock().await;
                ip.insert(key_clone.clone());
            }

            let _permit = sem_clone.acquire().await.unwrap();

            let url = format!(
                "https://tile.openstreetmap.org/{}/{}/{}.png",
                key_clone.z, key_clone.x, key_clone.y
            );

            let result = reqwest::Client::new()
                .get(&url)
                .header("User-Agent", "Howler/1.0 (wolf-tracking-app)")
                .send()
                .await;

            let resp = match result {
                Ok(r) => r,
                Err(e) => {
                    eprintln!(
                        "Tile fetch error ({}, {}, {}): {}",
                        key_clone.z, key_clone.x, key_clone.y, e
                    );
                    let mut ip = in_progress_clone.lock().await;
                    ip.remove(&key_clone);
                    let mut f = failed_clone.lock().await;
                    f.insert(key_clone);
                    return;
                }
            };

            if !resp.status().is_success() {
                eprintln!(
                    "Tile HTTP {} ({}, {}, {})",
                    resp.status(),
                    key_clone.z,
                    key_clone.x,
                    key_clone.y
                );
                let mut ip = in_progress_clone.lock().await;
                ip.remove(&key_clone);
                let mut f = failed_clone.lock().await;
                f.insert(key_clone);
                return;
            }

            let bytes = match resp.bytes().await {
                Ok(b) => b,
                Err(e) => {
                    eprintln!("Tile read error: {}", e);
                    let mut ip = in_progress_clone.lock().await;
                    ip.remove(&key_clone);
                    return;
                }
            };

            // Save to disk cache
            let path = disk_dir
                .join(key_clone.z.to_string())
                .join(key_clone.x.to_string())
                .join(format!("{}.png", key_clone.y));
            if let Some(parent) = path.parent() {
                let _ = std::fs::create_dir_all(parent);
            }
            let _ = std::fs::write(&path, &bytes);

            let image = match image::load_from_memory(&bytes) {
                Ok(img) => img.to_rgba8(),
                Err(e) => {
                    eprintln!("Tile decode error: {}", e);
                    let mut ip = in_progress_clone.lock().await;
                    ip.remove(&key_clone);
                    return;
                }
            };

            let size = [image.width() as usize, image.height() as usize];
            let pixels = image.into_raw();

            // Remove from failed set on success
            {
                let mut f = failed_clone.lock().await;
                f.remove(&key_clone);
            }

            ctx_clone.request_repaint();

            let mut cache = cache_clone.lock().await;
            let texture = ctx_clone.load_texture(
                format!("tile_{}_{}_{}", key_clone.z, key_clone.x, key_clone.y),
                egui::ColorImage::from_rgba_unmultiplied(size, &pixels),
                egui::TextureOptions::LINEAR,
            );

            cache.put(key_clone.clone(), TileData { texture });

            let mut ip = in_progress_clone.lock().await;
            ip.remove(&key_clone);
        });

        None
    }

    fn lat_lon_to_screen(&self, lat: f64, lon: f64, map_rect: egui::Rect) -> egui::Pos2 {
        let zoom = self.zoom_level();
        let n = 1 << zoom;

        // Convert lat/lon to tile coordinates at current zoom
        let tile_x = (lon + 180.0) / 360.0 * n as f64;
        let lat_rad = lat.to_radians();
        let tile_y = (1.0 - lat_rad.tan().ln() + std::f64::consts::PI / 2.0) / std::f64::consts::PI
            * n as f64;

        // Center tile coordinates (continuous)
        let center_tx_f = (self.center_lon + 180.0) / 360.0 * n as f64;
        let center_ty_f = {
            let lat_rad = self.center_lat.to_radians();
            (1.0 - lat_rad.tan().ln() + std::f64::consts::PI / 2.0) / std::f64::consts::PI
                * n as f64
        };

        let tile_size = 256.0_f64 * self.zoom as f64;

        let dx = (tile_x - center_tx_f) * tile_size + self.pan.x as f64;
        let dy = (tile_y - center_ty_f) * tile_size + self.pan.y as f64;

        egui::Pos2::new(
            map_rect.center().x + dx as f32,
            map_rect.center().y + dy as f32,
        )
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
                ui.checkbox(&mut self.offline_mode, "Offline Mode");
                ui.separator();
                ui.checkbox(&mut self.dark_mode, "Dark Mode");
                ui.separator();
                if self.offline_mode {
                    ui.label(
                        egui::RichText::new("(using cached tiles only)")
                            .color(egui::Color32::from_rgb(200, 150, 50)),
                    );
                } else {
                    let failed = self.failed_tiles.blocking_lock();
                    let count = failed.len();
                    drop(failed);
                    if count > 0 {
                        ui.label(
                            egui::RichText::new(format!("({} tiles unavailable)", count))
                                .color(egui::Color32::from_rgb(200, 80, 80)),
                        );
                    } else {
                        ui.label(
                            egui::RichText::new("(online)")
                                .color(egui::Color32::from_rgb(80, 180, 80)),
                        );
                    }
                }
            });

            ui.separator();

            // Map canvas
            let desired_size = egui::vec2(ui.available_width(), 400.0);
            let (map_rect, map_response) =
                ui.allocate_exact_size(desired_size, egui::Sense::click_and_drag());

            let painter = ui.painter_at(map_rect);

            // Draw OSM tiles using proper lat/lon -> tile math
            let zoom = self.zoom_level();
            let n = 1 << zoom;
            let tile_size = 256.0 * self.zoom;

            // Get center tile
            let (center_tx, center_ty) =
                self.lat_lon_to_tile(self.center_lat, self.center_lon, zoom);

            // How many tiles fit in the map area
            let tiles_across = (map_rect.width() / tile_size).ceil() as i32 + 2;
            let tiles_down = (map_rect.height() / tile_size).ceil() as i32 + 2;

            let half_w = tiles_across / 2;
            let half_h = tiles_down / 2;

            // Try to fetch failed tiles again (retry once per frame)
            let retry_keys: Vec<TileKey> = {
                let failed = self.failed_tiles.blocking_lock();
                failed.iter().cloned().collect()
            };
            for key in &retry_keys {
                self.get_or_fetch_tile(ctx, key.x, key.y, key.z);
            }

            for dx in -half_w..=half_w {
                for dy in -half_h..=half_h {
                    let tile_x = center_tx as i32 + dx;
                    let tile_y = center_ty as i32 + dy;

                    if tile_x < 0 || tile_y < 0 || tile_x >= n || tile_y >= n {
                        continue;
                    }

                    let tile_x = tile_x as u32;
                    let tile_y = tile_y as u32;

                    let screen_x = map_rect.center().x + (dx as f32 * tile_size) + self.pan.x;
                    let screen_y = map_rect.center().y + (dy as f32 * tile_size) + self.pan.y;

                    let tile_rect = egui::Rect::from_min_size(
                        egui::Pos2::new(screen_x, screen_y),
                        egui::vec2(tile_size, tile_size),
                    );

                    if tile_rect.intersects(map_rect) {
                        if let Some(texture) = self.get_or_fetch_tile(ctx, tile_x, tile_y, zoom) {
                            painter.image(
                                texture.id(),
                                tile_rect,
                                egui::Rect::from_min_max(
                                    egui::pos2(0.0, 0.0),
                                    egui::pos2(1.0, 1.0),
                                ),
                                egui::Color32::WHITE,
                            );
                        } else {
                            let bg = if self.dark_mode {
                                egui::Color32::from_rgb(50, 50, 50)
                            } else {
                                egui::Color32::from_rgb(230, 230, 230)
                            };
                            painter.rect_filled(tile_rect, 0.0, bg);
                        }
                    }
                }
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

            // Draw pack territories if enabled
            if self.show_pack_territories {
                painter.rect_stroke(
                    egui::Rect::from_min_size(
                        self.lat_lon_to_screen(44.5, -110.5, map_rect),
                        egui::vec2(100.0 * self.zoom, 80.0 * self.zoom),
                    ),
                    0.0,
                    (2.0, egui::Color32::from_rgb(255, 100, 100)),
                );
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
            ui.label(format!("Zoom: {:.1}x | Zoom level: {}", self.zoom, zoom));
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
