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

### ML Module

Machine learning for wolf behavior prediction and next-location forecasting.

```rust
use howler_core::ml::{
    BehaviorModel, BehaviorFeatures, BehaviorPrediction, BehaviorType,
    LocationPrediction, ActivityPrediction, ActivityPeriod,
};

// Create model (uses pre-trained defaults)
let model = BehaviorModel::new();

// Build features from sighting sequence
let features = BehaviorFeatures::from_sightings(&sightings);

// Predict behavior type
let prediction: BehaviorPrediction = model.predict_behavior(&features);
// prediction.behavior_type: BehaviorType enum
// prediction.confidence: 0.0 - 1.0
// prediction.probabilities: HashMap<BehaviorType, f64>

// Predict next location
let location: LocationPrediction = model.predict_next_location(&features);
// location.latitude, location.longitude
// location.time_horizon_hours
// location.confidence

// Predict activity pattern by time of day
let activity: ActivityPrediction = model.predict_activity_pattern(&sightings);
// activity.periods: Vec<ActivityPeriod> with hour, activity_level, classification

// Train custom classifier from labeled data
let mut model = BehaviorModel::new();
model.train_classifier(&training_features, &labels)?;

// Train location predictor from movement data
model.train_location_predictor(&features, &target_locations)?;
```

#### BehaviorType Variants

| Variant | Description |
|---------|-------------|
| `Stationary` | Wolf remains in small area (< 1km radius) |
| `Territorial` | Circular/area-restricted movement pattern |
| `Linear` | Directed linear movement (dispersal/migration) |
| `Random` | No discernible pattern |
| `CentralPlace` | Activity centered on a den site |

#### BehaviorFeatures

11 ML features extracted from sighting sequences:
- `speed_mean`, `speed_std` — movement speed statistics
- `direction_change_mean`, `direction_change_std` — turning angle stats
- `distance_from_start` — net displacement
- `radius_of_gyration` — spatial spread
- `sinuosity` — path tortuosity (0 = straight, 1 = circular)
- `time_span_hours` — observation duration
- `num_sightings` — sequence length
- `hour_of_day_mean`, `hour_of_day_std` — temporal distribution

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
