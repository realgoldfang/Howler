# Howler API Documentation

This document describes the public API provided by the `howler-core` library.

## Modules

### Config

Configuration management for API credentials.

```rust
use howler_core::Config;

// Load configuration from environment variables
let config = Config::from_env();

// Check for credentials
if config.has_movebank_credentials() {
    // Use Movebank API
}

if config.has_inaturalist_token() {
    // Use iNaturalist API
}

if config.has_iucn_token() {
    // Use IUCN API
}
```

#### Environment Variables

- `MOVEBANK_USERNAME`: Movebank username
- `MOVEBANK_PASSWORD`: Movebank password
- `INATURALIST_TOKEN`: iNaturalist API token
- `IUCN_TOKEN`: IUCN Red List API token

### Database

SQLite database for storing sightings and species status.

```rust
use howler_core::Database;

// Open or create database
let db = Database::new("howler.db")?;

// Insert a sighting
let id = db.insert_sighting(&sighting)?;

// Get all sightings
let sightings = db.get_all_sightings()?;

// Insert species status
let status_id = db.insert_species_status(&status)?;

// Get species status
let status = db.get_species_status("Canis lupus")?;
```

#### Methods

- `new(path: &str) -> Result<Database>`: Open or create database
- `insert_sighting(&self, sighting: &Sighting) -> Result<i64>`: Insert or replace a sighting
- `get_all_sightings(&self) -> Result<Vec<Sighting>>`: Retrieve all sightings
- `insert_species_status(&self, status: &SpeciesStatus) -> Result<i64>`: Insert or replace species status
- `get_species_status(&self, scientific_name: &str) -> Result<Option<SpeciesStatus>>`: Get species status by name

### Models

Data structures for sightings and species information.

#### Sighting

```rust
use howler_core::Sighting;
use howler_core::Source;
use chrono::Utc;

let sighting = Sighting {
    id: None,  // Set by database on insert
    species: "Canis lupus".to_string(),
    scientific_name: Some("Canis lupus".to_string()),
    latitude: 45.5,
    longitude: -122.5,
    observed_on: Utc::now(),
    source: Source::GBIF,
    source_id: "gbif:12345".to_string(),
    details: Some("Yellowstone National Park".to_string()),
};
```

**Fields:**
- `id: Option<i64>`: Database ID (None before insert)
- `species: String`: Common or species name
- `scientific_name: Option<String>`: Scientific name
- `latitude: f64`: Latitude coordinate
- `longitude: f64`: Longitude coordinate
- `observed_on: DateTime<Utc>`: Observation timestamp
- `source: Source`: Data source
- `source_id: String`: Unique ID from source
- `details: Option<String>`: Additional details

#### Source

```rust
use howler_core::Source;

let source = Source::GBIF;
println!("{}", source);  // Prints "GBIF"
```

**Variants:**
- `GBIF`: Global Biodiversity Information Facility
- `Movebank`: GPS tracking data
- `INaturalist`: Citizen science observations
- `IUCN`: Conservation status (not sightings)

#### SpeciesStatus

```rust
use howler_core::SpeciesStatus;

let status = SpeciesStatus {
    id: None,
    scientific_name: "Canis lupus".to_string(),
    common_name: Some("Gray Wolf".to_string()),
    red_list_category: Some("LC".to_string()),
    population_trend: Some("Stable".to_string()),
    threats: Some("Habitat loss".to_string()),
};
```

**Fields:**
- `id: Option<i64>`: Database ID
- `scientific_name: String`: Scientific name
- `common_name: Option<String>`: Common name
- `red_list_category: Option<String>`: IUCN Red List category
- `population_trend: Option<String>`: Population trend
- `threats: Option<String>`: Known threats

### GBIF Client

Fetch wolf sightings from GBIF API.

```rust
use howler_core::gbif::GBIFClient;

let client = GBIFClient::new();
let sightings = client.fetch_wolf_sightings(100).await?;
```

#### Methods

- `new() -> GBIFClient`: Create new client
- `fetch_wolf_sightings(&self, limit: u32) -> Result<Vec<Sighting>>`: Fetch wolf sightings

### iNaturalist Client

Fetch wolf sightings from iNaturalist API.

```rust
use howler_core::inaturalist::INaturalistClient;
use howler_core::Config;

let config = Config::from_env();
let client = INaturalistClient::new(&config);
let sightings = client.fetch_wolf_sightings(100).await?;
```

#### Methods

- `new(config: &Config) -> INaturalistClient`: Create new client with config
- `fetch_wolf_sightings(&self, limit: u32) -> Result<Vec<Sighting>>`: Fetch wolf sightings

### IUCN Client

Fetch species conservation status from IUCN API.

```rust
use howler_core::iucn::IUCNClient;
use howler_core::Config;

let config = Config::from_env();
let client = IUCNClient::new(&config);
let status = client.fetch_wolf_status().await?;
```

#### Methods

- `new(config: &Config) -> IUCNClient`: Create new client with config
- `fetch_species_status(&self, scientific_name: &str) -> Result<Option<SpeciesStatus>>`: Fetch species status
- `fetch_wolf_status(&self) -> Result<Option<SpeciesStatus>>`: Fetch wolf status specifically

### Movebank Client

Fetch GPS tracking data from Movebank API.

```rust
use howler_core::movebank::MovebankClient;
use howler_core::Config;

let config = Config::from_env();
if let Some(client) = MovebankClient::new(&config) {
    let sightings = client.fetch_wolf_tracks(100).await?;
}
```

#### Methods

- `new(config: &Config) -> Option<MovebankClient>`: Create new client (returns None if no credentials)
- `fetch_wolf_tracks(&self, limit: u32) -> Result<Vec<Sighting>>`: Fetch wolf GPS tracks

## Error Handling

All functions return `Result<T>` using the `anyhow` crate for error handling.

```rust
use anyhow::Result;

fn fetch_sightings() -> Result<Vec<Sighting>> {
    let db = Database::new("howler.db")?;
    let sightings = db.get_all_sightings()?;
    Ok(sightings)
}
```

## Example: Complete Workflow

```rust
use howler_core::{Config, Database, gbif};
use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    // Load configuration
    let config = Config::from_env();
    
    // Open database
    let db = Database::new("howler.db")?;
    
    // Fetch from GBIF
    let client = gbif::GBIFClient::new();
    let sightings = client.fetch_wolf_sightings(50).await?;
    
    // Store in database
    for sighting in sightings {
        db.insert_sighting(&sighting)?;
    }
    
    // Retrieve and display
    let all_sightings = db.get_all_sightings()?;
    println!("Total sightings: {}", all_sightings.len());
    
    Ok(())
}
```

## Testing

The library includes comprehensive unit and integration tests.

```bash
# Run all tests
cargo test -p howler-core

# Run specific test
cargo test -p howler-core test_database_creation

# Run with output
cargo test -p howler-core -- --nocapture
```
