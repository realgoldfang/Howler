use anyhow::{Context, Result};
use rusqlite::{params, Connection};

/// Database migration
pub struct Migration {
    pub version: i32,
    pub name: &'static str,
    pub sql: &'static str,
}

/// Get all migrations in order
pub fn get_migrations() -> Vec<Migration> {
    vec![
        Migration {
            version: 1,
            name: "initial_schema",
            sql: r#"
                CREATE TABLE IF NOT EXISTS sightings (
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
                );

                CREATE TABLE IF NOT EXISTS species_status (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    scientific_name TEXT NOT NULL UNIQUE,
                    common_name TEXT,
                    red_list_category TEXT,
                    population_trend TEXT,
                    threats TEXT
                );

                CREATE INDEX IF NOT EXISTS idx_sightings_species ON sightings(species);
                CREATE INDEX IF NOT EXISTS idx_sightings_source ON sightings(source);
            "#,
        },
        Migration {
            version: 2,
            name: "add_advanced_features",
            sql: r#"
                CREATE TABLE IF NOT EXISTS packs (
                    pack_id INTEGER PRIMARY KEY AUTOINCREMENT,
                    territory_geometry TEXT,
                    estimated_size INTEGER,
                    center_latitude REAL,
                    center_longitude REAL,
                    area_km2 REAL
                );

                CREATE TABLE IF NOT EXISTS individuals (
                    individual_id INTEGER PRIMARY KEY AUTOINCREMENT,
                    individual_identifier TEXT UNIQUE,
                    species TEXT,
                    sex TEXT,
                    age_class TEXT,
                    pack_id INTEGER,
                    FOREIGN KEY (pack_id) REFERENCES packs(pack_id)
                );

                CREATE TABLE IF NOT EXISTS movements (
                    movement_id INTEGER PRIMARY KEY AUTOINCREMENT,
                    from_sighting_id INTEGER,
                    to_sighting_id INTEGER,
                    distance_km REAL,
                    bearing_degrees REAL,
                    duration_seconds INTEGER,
                    speed_kmh REAL,
                    FOREIGN KEY (from_sighting_id) REFERENCES sightings(id),
                    FOREIGN KEY (to_sighting_id) REFERENCES sightings(id)
                );

                CREATE INDEX IF NOT EXISTS idx_sightings_observed_on ON sightings(observed_on);
                CREATE INDEX IF NOT EXISTS idx_sightings_location ON sightings(latitude, longitude);
                CREATE INDEX IF NOT EXISTS idx_individuals_pack ON individuals(pack_id);
            "#,
        },
        Migration {
            version: 3,
            name: "add_auth_and_annotations",
            sql: r#"
                CREATE TABLE IF NOT EXISTS users (
                    id TEXT PRIMARY KEY,
                    username TEXT NOT NULL UNIQUE,
                    email TEXT NOT NULL UNIQUE,
                    password_hash TEXT NOT NULL,
                    role TEXT NOT NULL DEFAULT 'Viewer',
                    created_at TEXT NOT NULL
                );

                CREATE TABLE IF NOT EXISTS sessions (
                    user_id TEXT NOT NULL,
                    token TEXT PRIMARY KEY,
                    expires_at TEXT NOT NULL,
                    FOREIGN KEY (user_id) REFERENCES users(id)
                );

                CREATE TABLE IF NOT EXISTS annotations (
                    id TEXT PRIMARY KEY,
                    sighting_id INTEGER NOT NULL,
                    user_id TEXT NOT NULL,
                    text TEXT NOT NULL,
                    annotation_type TEXT NOT NULL DEFAULT 'Comment',
                    created_at TEXT NOT NULL,
                    updated_at TEXT NOT NULL,
                    FOREIGN KEY (sighting_id) REFERENCES sightings(id),
                    FOREIGN KEY (user_id) REFERENCES users(id)
                );

                CREATE TABLE IF NOT EXISTS sighting_ratings (
                    id TEXT PRIMARY KEY,
                    sighting_id INTEGER NOT NULL,
                    user_id TEXT NOT NULL,
                    confidence INTEGER NOT NULL CHECK(confidence >= 1 AND confidence <= 5),
                    notes TEXT,
                    created_at TEXT NOT NULL,
                    FOREIGN KEY (sighting_id) REFERENCES sightings(id),
                    FOREIGN KEY (user_id) REFERENCES users(id),
                    UNIQUE(sighting_id, user_id)
                );

                CREATE INDEX IF NOT EXISTS idx_sessions_token ON sessions(token);
                CREATE INDEX IF NOT EXISTS idx_users_username ON users(username);
                CREATE INDEX IF NOT EXISTS idx_annotations_sighting ON annotations(sighting_id);
                CREATE INDEX IF NOT EXISTS idx_annotations_user ON annotations(user_id);
                CREATE INDEX IF NOT EXISTS idx_ratings_sighting ON sighting_ratings(sighting_id);
            "#,
        },
    ]
}

/// Create migrations table if it doesn't exist
fn ensure_migrations_table(conn: &Connection) -> Result<()> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS schema_migrations (
            version INTEGER PRIMARY KEY,
            applied_at TEXT NOT NULL
        )",
        [],
    )
    .context("Failed to create migrations table")?;
    Ok(())
}

/// Get current migration version
fn get_current_version(conn: &Connection) -> Result<i32> {
    let mut stmt = conn.prepare("SELECT MAX(version) FROM schema_migrations")?;
    let version: Option<i32> = stmt.query_row([], |row| row.get(0))?;
    Ok(version.unwrap_or(0))
}

/// Apply a single migration
fn apply_migration(conn: &Connection, migration: &Migration) -> Result<()> {
    conn.execute_batch(migration.sql)
        .context(format!("Failed to apply migration {}", migration.version))?;

    conn.execute(
        "INSERT INTO schema_migrations (version, applied_at) VALUES (?1, datetime('now'))",
        params![migration.version],
    )
    .context("Failed to record migration")?;

    Ok(())
}

/// Run all pending migrations
pub fn run_migrations(conn: &Connection) -> Result<()> {
    ensure_migrations_table(conn)?;
    let current_version = get_current_version(conn)?;
    let migrations = get_migrations();

    for migration in migrations {
        if migration.version > current_version {
            println!(
                "Applying migration v{}: {}",
                migration.version, migration.name
            );
            apply_migration(conn, &migration)?;
        }
    }

    Ok(())
}

/// Rollback to a specific version (destructive - use with caution)
pub fn rollback_to_version(conn: &Connection, target_version: i32) -> Result<()> {
    let current_version = get_current_version(conn)?;

    if target_version >= current_version {
        anyhow::bail!("Target version must be less than current version");
    }

    // For simplicity, we'll just delete the database and recreate to target version
    // In production, you'd want proper rollback migrations
    anyhow::bail!("Rollback not implemented - drop and recreate database instead");
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_get_migrations() {
        let migrations = get_migrations();
        assert_eq!(migrations.len(), 3);
        assert_eq!(migrations[0].version, 1);
        assert_eq!(migrations[1].version, 2);
        assert_eq!(migrations[2].version, 3);
    }

    #[test]
    fn test_run_migrations() {
        let temp_file = NamedTempFile::new().unwrap();
        let conn = Connection::open(temp_file.path()).unwrap();

        run_migrations(&conn).unwrap();

        let version = get_current_version(&conn).unwrap();
        assert_eq!(version, 3);
    }

    #[test]
    fn test_ensure_migrations_table() {
        let temp_file = NamedTempFile::new().unwrap();
        let conn = Connection::open(temp_file.path()).unwrap();

        ensure_migrations_table(&conn).unwrap();

        // Check table exists
        let mut stmt = conn
            .prepare(
                "SELECT name FROM sqlite_master WHERE type='table' AND name='schema_migrations'",
            )
            .unwrap();
        let result: Option<String> = stmt.query_row([], |row| row.get(0)).unwrap();
        assert_eq!(result, Some("schema_migrations".to_string()));
    }
}
