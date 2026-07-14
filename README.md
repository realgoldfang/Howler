# Howler

A production-ready desktop application for tracking and analyzing wolf (Canis lupus) sightings and movements across multiple data sources.

## Features

- **Multi-Source Data Integration**: Fetch wolf sightings from GBIF, iNaturalist, IUCN, and Movebank
- **Advanced Analysis**: Pack territory detection, movement analysis, and temporal pattern detection
- **Interactive Maps**: Full map capabilities with OpenStreetMap tiles, zoom/pan controls, and multiple layers
- **Multiple Interfaces**: CLI, TUI (Terminal UI), and GUI (Desktop) interfaces
- **Data Management**: Filter, search, export (CSV, GeoJSON, KML), and import wolf sighting data
- **Cross-Platform**: Runs on Linux, macOS, and Windows

## Installation

### From Source

```bash
# Clone the repository
git clone https://github.com/yourusername/howler.git
cd howler

# Build the project
cargo build --release

# The binaries will be in target/release/
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

### Viewing Data

```bash
# Launch TUI interface
howler-tui

# Launch GUI with map
howler-gui
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

## Development

### Prerequisites

- Rust 1.70 or later
- Cargo

### Building

```bash
# Build all components
cargo build --workspace

# Build specific component
cargo build -p howler-core
cargo build -p howler-cli
cargo build -p howler-tui
cargo build -p howler-gui
```

### Testing

```bash
# Run all tests
cargo test --workspace

# Run tests for specific component
cargo test -p howler-core

# Run with coverage
cargo tarpaulin --workspace
```

### Project Structure

```
howler/
├── howler-core/      # Core library with data models and API clients
├── howler-cli/       # Command-line interface
├── howler-tui/       # Terminal user interface
├── howler-gui/       # Graphical user interface
├── fixtures/         # Test fixtures
├── docs/             # Documentation
└── .github/          # CI/CD workflows
```

## Advanced Features

### Pack Territory Detection

Howler uses DBSCAN clustering to detect wolf pack territories from GPS coordinates:

```bash
# Run territory detection
howler-cli --analyze --territories
```

### Movement Analysis

Analyze wolf movement patterns, speeds, and directions:

```bash
# Analyze movement
howler-cli --analyze --movement
```

### Temporal Analysis

Detect activity patterns by time of day and seasonal distributions:

```bash
# Analyze temporal patterns
howler-cli --analyze --temporal
```

## Contributing

Contributions are welcome! Please follow these guidelines:

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Make your changes
4. Run tests (`cargo test`)
5. Run linters (`cargo fmt && cargo clippy`)
6. Commit your changes (`git commit -m 'Add amazing feature'`)
7. Push to the branch (`git push origin feature/amazing-feature`)
8. Open a Pull Request

### Code Style

- Use `cargo fmt` for formatting
- Use `cargo clippy` for linting
- Write tests for new features
- Update documentation as needed

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

## Support

- **Issues**: [GitHub Issues](https://github.com/yourusername/howler/issues)
- **Documentation**: [docs/](docs/)
- **Discussions**: [GitHub Discussions](https://github.com/yourusername/howler/discussions)

## Roadmap

- [ ] Real-time data streaming
- [ ] Machine learning for behavior prediction
- [ ] Mobile applications (iOS, Android)
- [ ] Web application
- [ ] Multi-user collaboration features
