use crate::models::{Sighting, Source};
use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use csv::ReaderBuilder;
use std::fs::File;
use std::io::Read;

/// Import sightings from CSV file
pub fn import_csv(file_path: &str) -> Result<Vec<Sighting>> {
    let file = File::open(file_path).context("Failed to open CSV file")?;
    let mut rdr = ReaderBuilder::new().from_reader(file);

    let mut sightings = Vec::new();

    for result in rdr.records() {
        let record = result.context("Failed to read CSV record")?;

        let species = record.get(1).unwrap_or("Unknown").to_string();
        let scientific_name = record.get(2).map(|s| s.to_string());
        let latitude: f64 = record
            .get(3)
            .unwrap_or("0")
            .parse()
            .context("Failed to parse latitude")?;
        let longitude: f64 = record
            .get(4)
            .unwrap_or("0")
            .parse()
            .context("Failed to parse longitude")?;
        let observed_on_str = record.get(5).unwrap_or("");
        let source_str = record.get(6).unwrap_or("GBIF");
        let source_id = record.get(7).unwrap_or("").to_string();
        let details = record.get(8).map(|s| s.to_string());

        let source = match source_str {
            "Movebank" => Source::Movebank,
            "iNaturalist" => Source::INaturalist,
            "IUCN" => Source::IUCN,
            _ => Source::GBIF,
        };

        let observed_on = DateTime::parse_from_rfc3339(observed_on_str)
            .map(|dt| dt.with_timezone(&Utc))
            .unwrap_or_else(|_| Utc::now());

        sightings.push(Sighting {
            id: None,
            species,
            scientific_name,
            latitude,
            longitude,
            observed_on,
            source,
            source_id,
            details,
        });
    }

    Ok(sightings)
}

/// Import sightings from GeoJSON file
pub fn import_geojson(file_path: &str) -> Result<Vec<Sighting>> {
    let mut file = File::open(file_path).context("Failed to open GeoJSON file")?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)
        .context("Failed to read GeoJSON file")?;

    let geojson: geojson::GeoJson =
        serde_json::from_str(&contents).context("Failed to parse GeoJSON")?;

    let mut sightings = Vec::new();

    if let geojson::GeoJson::FeatureCollection(collection) = geojson {
        for feature in collection.features {
            if let Some(geometry) = feature.geometry {
                if let geojson::Value::Point(coords) = geometry.value {
                    if coords.len() >= 2 {
                        let longitude = coords[0];
                        let latitude = coords[1];

                        let properties = feature.properties.unwrap_or_default();
                        let species = properties
                            .get("species")
                            .and_then(|v| v.as_str())
                            .unwrap_or("Unknown")
                            .to_string();
                        let scientific_name = properties
                            .get("scientific_name")
                            .and_then(|v| v.as_str())
                            .map(|s| s.to_string());
                        let observed_on_str = properties
                            .get("observed_on")
                            .and_then(|v| v.as_str())
                            .unwrap_or("");
                        let source_str = properties
                            .get("source")
                            .and_then(|v| v.as_str())
                            .unwrap_or("GBIF");
                        let source_id = properties
                            .get("source_id")
                            .and_then(|v| v.as_str())
                            .unwrap_or("")
                            .to_string();
                        let details = properties
                            .get("details")
                            .and_then(|v| v.as_str())
                            .map(|s| s.to_string());

                        let source = match source_str {
                            "Movebank" => Source::Movebank,
                            "iNaturalist" => Source::INaturalist,
                            "IUCN" => Source::IUCN,
                            _ => Source::GBIF,
                        };

                        let observed_on = DateTime::parse_from_rfc3339(observed_on_str)
                            .map(|dt| dt.with_timezone(&Utc))
                            .unwrap_or_else(|_| Utc::now());

                        sightings.push(Sighting {
                            id: None,
                            species,
                            scientific_name,
                            latitude,
                            longitude,
                            observed_on,
                            source,
                            source_id,
                            details,
                        });
                    }
                }
            }
        }
    }

    Ok(sightings)
}

/// Validate sighting data
pub fn validate_sighting(sighting: &Sighting) -> Result<()> {
    if sighting.latitude < -90.0 || sighting.latitude > 90.0 {
        anyhow::bail!("Invalid latitude: {}", sighting.latitude);
    }

    if sighting.longitude < -180.0 || sighting.longitude > 180.0 {
        anyhow::bail!("Invalid longitude: {}", sighting.longitude);
    }

    if sighting.species.is_empty() {
        anyhow::bail!("Species cannot be empty");
    }

    if sighting.source_id.is_empty() {
        anyhow::bail!("Source ID cannot be empty");
    }

    Ok(())
}

/// Import and validate sightings
pub fn import_and_validate(
    file_path: &str,
    format: super::export::ExportFormat,
) -> Result<Vec<Sighting>> {
    let sightings = match format {
        super::export::ExportFormat::Csv => import_csv(file_path)?,
        super::export::ExportFormat::GeoJson => import_geojson(file_path)?,
        super::export::ExportFormat::Kml => {
            anyhow::bail!("KML import not yet implemented");
        }
    };

    // Validate all sightings
    for sighting in &sightings {
        validate_sighting(sighting)?;
    }

    Ok(sightings)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_sighting_valid() {
        let sighting = Sighting {
            id: None,
            species: "Canis lupus".to_string(),
            scientific_name: Some("Canis lupus".to_string()),
            latitude: 45.0,
            longitude: -122.0,
            observed_on: Utc::now(),
            source: Source::GBIF,
            source_id: "test_123".to_string(),
            details: None,
        };

        assert!(validate_sighting(&sighting).is_ok());
    }

    #[test]
    fn test_validate_sighting_invalid_lat() {
        let sighting = Sighting {
            id: None,
            species: "Canis lupus".to_string(),
            scientific_name: Some("Canis lupus".to_string()),
            latitude: 95.0,
            longitude: -122.0,
            observed_on: Utc::now(),
            source: Source::GBIF,
            source_id: "test_123".to_string(),
            details: None,
        };

        assert!(validate_sighting(&sighting).is_err());
    }

    #[test]
    fn test_validate_sighting_empty_species() {
        let sighting = Sighting {
            id: None,
            species: "".to_string(),
            scientific_name: Some("Canis lupus".to_string()),
            latitude: 45.0,
            longitude: -122.0,
            observed_on: Utc::now(),
            source: Source::GBIF,
            source_id: "test_123".to_string(),
            details: None,
        };

        assert!(validate_sighting(&sighting).is_err());
    }
}
