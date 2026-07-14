# Howler

A production-ready application for tracking and analyzing wolf (Canis lupus) sightings and movements across multiple data sources — with ML-powered behavior prediction.

## Features

- **Multi-Source Data Integration**: Fetch wolf sightings from GBIF, iNaturalist, IUCN, and Movebank
- **Advanced Analysis**: Pack territory detection (DBSCAN), movement analysis, and temporal pattern detection
- **Machine Learning**: Behavior prediction (stationary, territorial, linear, random, central-place), next-location forecasting, and activity pattern analysis
- **Interactive Maps**: Full map capabilities with OpenStreetMap tiles, zoom/pan controls, and multiple layers
- **Multiple Interfaces**: CLI, TUI (Terminal UI), GUI (Desktop), Web App, and Mobile App
- **Data Management**: Filter, search, export (CSV, GeoJSON, KML), and import wolf sighting data
- **Cross-Platform**: Runs on Linux, macOS, Windows, iOS, and Android

## Installation

### From Source

```bash
git clone https://github.com/yourusername/howler.git
cd howler
cargo build --release
```

### Using Cargo

```bash
cargo install howler-cli
cargo install howler-tui
cargo install howler-gui
```

### Pre-built Binaries

Download pre-built binaries from the [Releases](https://github.com/yourusername/howler/releases) page.

## Quick Start

### CLI

```bash
# Fetch wolf sightings from all sources
howler-cli --fetch

# Generate a report
howler-cli --report

# Fetch from specific source
howler-cli --fetch --source gbif --limit 100
```

### TUI (Terminal UI)

```bash
howler-tui
```

### GUI (Desktop)

```bash
howler-gui
```

### Web App

```bash
cd web-app
npm install
npm run dev
```

Opens at `http://localhost:5173` with dashboard, map, analysis, ML predictions, and settings.

### Mobile App (React Native / Expo)

```bash
cd mobile-app
npm install
npx expo start
```

Scan the QR code with Expo Go on iOS or Android.

## Configuration

Howler uses environment variables for API key configuration:

```bash
# Movebank credentials (for GPS tracking data)
export MOVEBANK_USERNAME="your_username"
export MOVEBANK_PASSWORD="your_password"

# iNaturalist API token (for citizen science observations)
export INATURALIST_TOKEN="your_token"

# IUCN API token (for conservation status)
export IUCN_TOKEN="your_token"
```

### API Key Acquisition

**Movebank**:
1. Register at [movebank.org](https://www.movebank.org)
2. Create an account and request access to studies
3. Use your username and password

**iNaturalist**:
1. Register at [inaturalist.org](https://www.inaturalist.org)
2. Go to your account settings and create an API application
3. Copy the access token

**IUCN**:
1. Register at [api.iucnredlist.org](https://apiv3.iucnredlist.org)
2. Request an API token
3. Use the token in your environment

## Usage Examples

### Fetching Data

```bash
# Fetch from all sources
howler-cli --fetch

# Fetch specific number of sightings
howler-cli --fetch --limit 50

# Fetch from specific source
howler-cli --fetch --source gbif
```

### Exporting Data

```bash
# Export to CSV
howler-cli --export --format csv --output sightings.csv

# Export to GeoJSON
howler-cli --export --format geojson --output sightings.geojson

# Export to KML (Google Earth)
howler-cli --export --format kml --output sightings.kml
```

### Filtering Data

```bash
# Filter by date range
howler-cli --filter --start-date 2023-01-01 --end-date 2023-12-31

# Filter by source
howler-cli --filter --source gbif

# Filter by species
howler-cli --filter --species "Canis lupus"
```

## Machine Learning

Howler includes an ML module for wolf behavior prediction using random forest classification and linear regression.

### Behavior Classification

```rust
use howler_core::ml::{BehaviorModel, BehaviorFeatures, BehaviorType};

let model = BehaviorModel::new();
let features = BehaviorFeatures::from_sightings(&sightings);
let prediction = model.predict_behavior(&features);

match prediction.behavior_type {
    BehaviorType::Territorial => println!("Pack territory behavior"),
    BehaviorType::Linear => println!("Dispersal/migration pattern"),
    BehaviorType::CentralPlace => println!("Den-based activity"),
    // ...
}
```

### Next-Location Prediction

```rust
let location_pred = model.predict_next_location(&features);
println!(
    "Predicted: ({}, {}) in {} hours",
    location_pred.latitude, location_pred.longitude, location_pred.time_horizon_hours
);
```

### Supported Behavior Types

| Type | Description |
|------|-------------|
| `Stationary` | Wolf remains in small area (< 1km radius) |
| `Territorial` | Circular/area-restricted movement |
| `Linear` | Directed movement (dispersal/migration) |
| `Random` | No discernible pattern |
| `CentralPlace` | Activity centered on den site |

## Project Structure

```
howler/
├── howler-core/      # Core library: data models, API clients, ML, analysis
├── howler-cli/       # Command-line interface
├── howler-tui/       # Terminal user interface (Ratatui)
├── howler-gui/       # Desktop GUI (Iced)
├── web-app/          # React + Vite web application
├── mobile-app/       # React Native (Expo) mobile application
├── shared/           # Shared TypeScript types and API client
├── fixtures/         # Test fixtures
├── docs/             # Documentation
└── .github/          # CI/CD workflows
```

## Development

### Prerequisites

- Rust 1.70 or later
- Node.js 18+ (for web/mobile apps)
- Expo CLI (for mobile app)

### Building

```bash
# Build all Rust crates
cargo build --workspace

# Build web app
cd web-app && npm install && npm run build

# Build mobile app
cd mobile-app && npm install && npx expo start
```

### Testing

```bash
# Run all Rust tests
cargo test --all-features --workspace

# Run with coverage
cargo tarpaulin --workspace
```

### Code Quality

```bash
# Format
cargo fmt

# Lint
cargo clippy -D warnings
```

## Contributing

Contributions are welcome! Please follow these guidelines:

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Make your changes
4. Run tests (`cargo test --all-features --workspace`)
5. Run linters (`cargo fmt && cargo clippy -D warnings`)
6. Commit your changes (`git commit -m 'Add amazing feature'`)
7. Push to the branch (`git push origin feature/amazing-feature`)
8. Open a Pull Request

## License

This project is licensed under either of:

- MIT License ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)
- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)

## Data Sources

- **GBIF**: Global Biodiversity Information Facility - https://www.gbif.org
- **iNaturalist**: Citizen science observations - https://www.inaturalist.org
- **IUCN**: International Union for Conservation of Nature - https://www.iucnredlist.org
- **Movebank**: Animal tracking data - https://www.movebank.org

## Acknowledgments

- GBIF for providing occurrence data
- iNaturalist for citizen science observations
- IUCN for conservation status data
- Movebank for GPS tracking data
- The Rust community for excellent tools and libraries
- linfa for machine learning primitives

## Support

- **Issues**: [GitHub Issues](https://github.com/yourusername/howler/issues)
- **Documentation**: [docs/](docs/)
- **Discussions**: [GitHub Discussions](https://github.com/yourusername/howler/discussions)

## Roadmap

- [x] Multi-source data integration (GBIF, iNaturalist, IUCN, Movebank)
- [x] DBSCAN territory detection
- [x] Movement and temporal analysis
- [x] CLI, TUI, and GUI interfaces
- [x] Data export (CSV, GeoJSON, KML) and import
- [x] Machine learning for behavior prediction
- [x] Web application (React + Vite)
- [x] Mobile application (React Native / Expo)
- [ ] Real-time data streaming
- [ ] Multi-user collaboration features
- [ ] Offline map tiles for mobile
