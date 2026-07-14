use crate::models::{Sighting, Source, SpeciesStatus};
use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use rusqlite::{params, Connection};
use std::sync::Mutex;

pub struct Database {
    conn: Mutex<Connection>,
}

impl Database {
    pub fn new(path: &str) -> Result<Self> {
        let conn = Connection::open(path).context("Failed to open database")?;

        let db = Database {
            conn: Mutex::new(conn),
        };
        db.init_schema()?;
        Ok(db)
    }

    fn init_schema(&self) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "CREATE TABLE IF NOT EXISTS sightings (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                species TEXT NOT NULL,
                scientific_name TEXT,
                latitude REAL NOT NULL,
                longitude REAL NOT NULL,
                observed_on TEXT NOT NULL,
                source TEXT NOT NULL,
                source_id TEXT NOT NULL,
                details TEXT,
                UNIQUE(source, source_id)
            )",
            [],
        )
        .context("Failed to create sightings table")?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS species_status (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                scientific_name TEXT NOT NULL UNIQUE,
                common_name TEXT,
                red_list_category TEXT,
                population_trend TEXT,
                threats TEXT
            )",
            [],
        )
        .context("Failed to create species_status table")?;

        // New tables for advanced features
        conn.execute(
            "CREATE TABLE IF NOT EXISTS packs (
                pack_id INTEGER PRIMARY KEY AUTOINCREMENT,
                territory_geometry TEXT,
                estimated_size INTEGER,
                center_latitude REAL,
                center_longitude REAL,
                area_km2 REAL
            )",
            [],
        )
        .context("Failed to create packs table")?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS individuals (
                individual_id INTEGER PRIMARY KEY AUTOINCREMENT,
                individual_identifier TEXT UNIQUE,
                species TEXT,
                sex TEXT,
                age_class TEXT,
                pack_id INTEGER,
                FOREIGN KEY (pack_id) REFERENCES packs(pack_id)
            )",
            [],
        )
        .context("Failed to create individuals table")?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS movements (
                movement_id INTEGER PRIMARY KEY AUTOINCREMENT,
                from_sighting_id INTEGER,
                to_sighting_id INTEGER,
                distance_km REAL,
                bearing_degrees REAL,
                duration_seconds INTEGER,
                speed_kmh REAL,
                FOREIGN KEY (from_sighting_id) REFERENCES sightings(id),
                FOREIGN KEY (to_sighting_id) REFERENCES sightings(id)
            )",
            [],
        )
        .context("Failed to create movements table")?;

        // Indexes for performance
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_sightings_species ON sightings(species)",
            [],
        )
        .context("Failed to create species index")?;

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_sightings_source ON sightings(source)",
            [],
        )
        .context("Failed to create source index")?;

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_sightings_observed_on ON sightings(observed_on)",
            [],
        )
        .context("Failed to create observed_on index")?;

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_sightings_location ON sightings(latitude, longitude)",
            [],
        )
        .context("Failed to create location index")?;

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_individuals_pack ON individuals(pack_id)",
            [],
        )
        .context("Failed to create individuals pack index")?;

        // Performance optimizations
        let _ = conn.query_row("PRAGMA journal_mode = WAL", [], |row| {
            row.get::<_, String>(0)
        });
        let _ = conn.execute("PRAGMA synchronous = NORMAL", []);
        let _ = conn.execute("PRAGMA cache_size = -64000", []);

        Ok(())
    }

    pub fn insert_sighting(&self, sighting: &Sighting) -> Result<i64> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT OR REPLACE INTO sightings 
             (species, scientific_name, latitude, longitude, observed_on, source, source_id, details)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                sighting.species,
                sighting.scientific_name,
                sighting.latitude,
                sighting.longitude,
                sighting.observed_on.to_rfc3339(),
                sighting.source.to_string(),
                sighting.source_id,
                sighting.details,
            ],
        )
        .context("Failed to insert sighting")?;

        Ok(conn.last_insert_rowid())
    }

    /// Batch insert sightings for better performance
    pub fn insert_sightings_batch(&self, sightings: &[Sighting]) -> Result<usize> {
        let mut conn = self.conn.lock().unwrap();
        let tx = conn.transaction().context("Failed to begin transaction")?;

        let mut count = 0;
        for sighting in sightings {
            tx.execute(
                "INSERT OR REPLACE INTO sightings 
                 (species, scientific_name, latitude, longitude, observed_on, source, source_id, details)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
                params![
                    sighting.species,
                    sighting.scientific_name,
                    sighting.latitude,
                    sighting.longitude,
                    sighting.observed_on.to_rfc3339(),
                    sighting.source.to_string(),
                    sighting.source_id,
                    sighting.details,
                ],
            )
            .context("Failed to insert sighting in batch")?;
            count += 1;
        }

        tx.commit().context("Failed to commit transaction")?;
        Ok(count)
    }

    pub fn get_all_sightings(&self) -> Result<Vec<Sighting>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn
            .prepare(
                "SELECT id, species, scientific_name, latitude, longitude, observed_on, source, source_id, details
                 FROM sightings",
            )
            .context("Failed to prepare query")?;

        let sightings = stmt
            .query_map([], |row| {
                let observed_on_str: String = row.get(5)?;
                let observed_on = DateTime::parse_from_rfc3339(&observed_on_str)
                    .map(|dt| dt.with_timezone(&Utc))
                    .unwrap_or_else(|_| Utc::now());

                let source_str: String = row.get(6)?;
                let source = match source_str.as_str() {
                    "GBIF" => Source::GBIF,
                    "Movebank" => Source::Movebank,
                    "iNaturalist" => Source::INaturalist,
                    "IUCN" => Source::IUCN,
                    _ => Source::GBIF,
                };

                Ok(Sighting {
                    id: Some(row.get(0)?),
                    species: row.get(1)?,
                    scientific_name: row.get(2)?,
                    latitude: row.get(3)?,
                    longitude: row.get(4)?,
                    observed_on,
                    source,
                    source_id: row.get(7)?,
                    details: row.get(8)?,
                })
            })
            .context("Failed to execute query")?;

        let mut result = Vec::new();
        for sighting in sightings {
            result.push(sighting.context("Failed to parse sighting")?);
        }
        Ok(result)
    }

    pub fn insert_species_status(&self, status: &SpeciesStatus) -> Result<i64> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT OR REPLACE INTO species_status 
             (scientific_name, common_name, red_list_category, population_trend, threats)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![
                status.scientific_name,
                status.common_name,
                status.red_list_category,
                status.population_trend,
                status.threats,
            ],
        )
        .context("Failed to insert species status")?;

        Ok(conn.last_insert_rowid())
    }

    pub fn get_species_status(&self, scientific_name: &str) -> Result<Option<SpeciesStatus>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn
            .prepare(
                "SELECT id, scientific_name, common_name, red_list_category, population_trend, threats
                 FROM species_status WHERE scientific_name = ?1",
            )
            .context("Failed to prepare query")?;

        let mut result = None;
        let mut rows = stmt.query(params![scientific_name])?;

        if let Some(row) = rows.next()? {
            result = Some(SpeciesStatus {
                id: Some(row.get(0)?),
                scientific_name: row.get(1)?,
                common_name: row.get(2)?,
                red_list_category: row.get(3)?,
                population_trend: row.get(4)?,
                threats: row.get(5)?,
            });
        }

        Ok(result)
    }

    /// Get a reference to the underlying connection for auth/annotation modules
    pub fn connection(&self) -> std::sync::MutexGuard<'_, Connection> {
        self.conn.lock().unwrap()
    }
}
