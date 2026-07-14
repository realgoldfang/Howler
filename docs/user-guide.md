# Howler User Guide

This guide provides detailed instructions for using Howler's various interfaces and features.

## Table of Contents

- [CLI Reference](#cli-reference)
- [TUI Guide](#tui-guide)
- [GUI Guide](#gui-guide)
- [Data Interpretation](#data-interpretation)

## CLI Reference

### Basic Commands

#### Fetch Data

```bash
howler-cli --fetch
```

Fetch wolf sightings from all configured data sources.

**Options:**
- `--source <SOURCE>`: Fetch from specific source (gbif, inaturalist, iucn, movebank)
- `--limit <N>`: Limit number of sightings to fetch (default: 100)

**Examples:**
```bash
# Fetch from GBIF only
howler-cli --fetch --source gbif --limit 50

# Fetch from iNaturalist
howler-cli --fetch --source inaturalist --limit 200
```

#### Generate Report

```bash
howler-cli --report
```

Generate a summary report of all sightings in the database.

**Output includes:**
- Total number of sightings
- Breakdown by source
- Date range of sightings
- Geographic distribution summary

#### Export Data

```bash
howler-cli --export --format <FORMAT> --output <FILE>
```

Export sightings to various formats.

**Supported formats:**
- `csv`: Comma-separated values
- `geojson`: GeoJSON for mapping applications
- `kml`: KML for Google Earth

**Examples:**
```bash
# Export to CSV
howler-cli --export --format csv --output sightings.csv

# Export to GeoJSON
howler-cli --export --format geojson --output sightings.geojson

# Export to KML
howler-cli --export --format kml --output sightings.kml
```

#### Filter Data

```bash
howler-cli --filter [OPTIONS]
```

Filter sightings before export or analysis.

**Options:**
- `--start-date <DATE>`: Start date (YYYY-MM-DD)
- `--end-date <DATE>`: End date (YYYY-MM-DD)
- `--source <SOURCE>`: Filter by data source
- `--species <SPECIES>`: Filter by species name

**Examples:**
```bash
# Filter by date range
howler-cli --filter --start-date 2023-01-01 --end-date 2023-12-31

# Filter by source
howler-cli --filter --source gbif

# Combine filters
howler-cli --filter --start-date 2023-01-01 --source inaturalist
```

#### Analyze Data

```bash
howler-cli --analyze <TYPE>
```

Run advanced analysis on sighting data.

**Analysis types:**
- `territories`: Detect pack territories using DBSCAN clustering
- `movement`: Analyze movement patterns and speeds
- `temporal`: Analyze temporal patterns by time of day and season

**Examples:**
```bash
# Detect pack territories
howler-cli --analyze territories

# Analyze movement patterns
howler-cli --analyze movement

# Analyze temporal patterns
howler-cli --analyze temporal
```

#### Import Data

```bash
howler-cli --import --format <FORMAT> --input <FILE>
```

Import sightings from external files.

**Supported formats:**
- `csv`: Comma-separated values
- `geojson`: GeoJSON format

**Examples:**
```bash
# Import from CSV
howler-cli --import --format csv --input my_sightings.csv

# Import from GeoJSON
howler-cli --import --format geojson --input my_sightings.geojson
```

#### Cleanup Data

```bash
howler-cli --cleanup [OPTIONS]
```

Clean up the database.

**Options:**
- `--duplicates`: Remove duplicate sightings
- `--out-of-range`: Remove sightings with invalid coordinates
- `--old-data <DAYS>`: Remove data older than specified days
- `--vacuum`: Vacuum the database to reclaim space

**Examples:**
```bash
# Remove duplicates
howler-cli --cleanup --duplicates

# Remove old data (older than 365 days)
howler-cli --cleanup --old-data 365

# Vacuum database
howler-cli --cleanup --vacuum
```

### TUI Interface

#### Launching TUI

```bash
howler-tui
```

#### Keyboard Shortcuts

**Navigation:**
- `↑`/`↓`: Move up/down through list
- `PgUp`/`PgDn`: Page up/down
- `Home`/`End`: Jump to start/end
- `Enter`: Select item
- `Esc`: Go back/exit

**Views:**
- `1`: Sighting list
- `2`: Map view
- `3`: Statistics
- `4`: Settings
- `q`: Quit

**Actions:**
- `f`: Filter sightings
- `e`: Export data
- `r`: Refresh data
- `?`: Show help

#### Map Controls (TUI)

- `+`/`-`: Zoom in/out
- `Arrow keys`: Pan map
- `c`: Center on selected sighting
- `a`: Zoom to fit all sightings

### GUI Interface

#### Launching GUI

```bash
howler-gui
```

#### Main Window

The GUI consists of several panels:

1. **Sidebar**: Navigation and controls
2. **Map View**: Interactive map with sightings
3. **Sighting List**: Table of all sightings
4. **Details Panel**: Information about selected sighting

#### Map Controls

**Zoom:**
- Mouse wheel: Zoom in/out
- `+`/`-` buttons: Zoom in/out
- Right-click → Zoom to fit: Fit all sightings

**Pan:**
- Click and drag: Pan map
- Arrow keys: Pan map

**Layers:**
- Toggle tile sources (OSM, Satellite, Terrain)
- Toggle sighting markers by source
- Toggle territory boundaries
- Toggle movement paths

**Drawing Tools:**
- Draw custom territories (polygons)
- Draw routes (polylines)
- Add custom markers

#### Exporting from GUI

1. Click "Export" button in toolbar
2. Select format (CSV, GeoJSON, KML, PNG)
3. Choose output location
4. Click "Export"

## Data Interpretation

### Understanding Sighting Sources

**GBIF (Global Biodiversity Information Facility):**
- Museum specimens and observations
- Historical records
- Varying accuracy and completeness
- Good for historical distribution analysis

**iNaturalist:**
- Citizen science observations
- Recent observations with photos
- Community-verified identifications
- Good for current distribution and activity patterns

**IUCN:**
- Conservation status data
- Population trends
- Threat assessments
- Not sighting data, but status information

**Movebank:**
- GPS tracking data from collared wolves
- High-precision movement data
- Individual wolf tracking
- Best for movement analysis and territory mapping

### Interpreting Territory Analysis

**DBSCAN Clustering:**
- Groups nearby GPS points into clusters
- Each cluster represents a potential pack territory
- Cluster size indicates territory extent
- Dense clusters indicate core areas

**Territory Overlap:**
- Overlapping territories may indicate:
  - Pack interactions
  - Shared resources
  - Territory disputes

### Interpreting Movement Analysis

**Speed:**
- Average speed: Normal movement patterns
- High speeds: Travel between territories
- Low speeds: Foraging or resting

**Direction:**
- Bearing calculations show movement direction
- Circular patterns: Territory patrolling
- Linear patterns: Migration or dispersal

**Movement Patterns:**
- Random: Exploratory behavior
- Circular: Territory-based movement
- Linear: Dispersal or migration

### Interpreting Temporal Analysis

**Time of Day:**
- Diurnal: Active during day
- Nocturnal: Active at night
- Crepuscular: Active at dawn/dusk

**Seasonal Patterns:**
- Breeding season: Increased territorial behavior
- Winter: Pack hunting, larger ranges
- Summer: Smaller ranges, pup rearing

## Troubleshooting

### Common Issues

**No data fetched:**
- Check API keys are set correctly
- Verify network connectivity
- Check API rate limits

**Map not displaying:**
- Ensure network connectivity for tile loading
- Check firewall settings
- Try different tile source

**Database errors:**
- Check file permissions
- Ensure sufficient disk space
- Try running database vacuum

### Getting Help

- Check the [README](../README.md) for basic setup
- Review [API documentation](api-guide.md) for technical details
- Open an issue on GitHub for bugs
- Join discussions for questions
