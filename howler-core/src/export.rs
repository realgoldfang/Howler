use crate::models::Sighting;
use anyhow::Result;
use std::fs::File;
use std::io::Write;

/// Export format
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ExportFormat {
    Csv,
    GeoJson,
    Kml,
}

/// Export sightings to specified format
pub fn export_sightings(
    sightings: &[Sighting],
    format: ExportFormat,
    output_path: &str,
) -> Result<()> {
    match format {
        ExportFormat::Csv => export_csv(sightings, output_path),
        ExportFormat::GeoJson => export_geojson(sightings, output_path),
        ExportFormat::Kml => export_kml(sightings, output_path),
    }
}

/// Export sightings to CSV format
pub fn export_csv(sightings: &[Sighting], output_path: &str) -> Result<()> {
    let mut file = File::create(output_path)?;

    // Write header
    writeln!(
        file,
        "id,species,scientific_name,latitude,longitude,observed_on,source,source_id,details"
    )?;

    // Write data rows
    for sighting in sightings {
        let id = sighting.id.map(|i| i.to_string()).unwrap_or_default();
        let scientific_name = sighting.scientific_name.as_deref().unwrap_or("");
        let observed_on = sighting.observed_on.to_rfc3339();
        let details = sighting.details.as_deref().unwrap_or("");

        writeln!(
            file,
            "{},{},{},{},{},{},{},{},{}",
            id,
            sighting.species,
            scientific_name,
            sighting.latitude,
            sighting.longitude,
            observed_on,
            sighting.source,
            sighting.source_id,
            details
        )?;
    }

    Ok(())
}

/// Export sightings to GeoJSON format
pub fn export_geojson(sightings: &[Sighting], output_path: &str) -> Result<()> {
    use geojson::{Feature, FeatureCollection, Geometry, Value};

    let mut features = Vec::new();

    for sighting in sightings {
        let coords = vec![sighting.longitude, sighting.latitude];
        let geometry = Geometry::new(Value::Point(coords));

        let mut props = serde_json::Map::new();
        props.insert("id".to_string(), serde_json::json!(sighting.id));
        props.insert("species".to_string(), serde_json::json!(sighting.species));
        props.insert(
            "scientific_name".to_string(),
            serde_json::json!(sighting.scientific_name),
        );
        props.insert(
            "observed_on".to_string(),
            serde_json::json!(sighting.observed_on.to_rfc3339()),
        );
        props.insert(
            "source".to_string(),
            serde_json::json!(sighting.source.to_string()),
        );
        props.insert(
            "source_id".to_string(),
            serde_json::json!(sighting.source_id),
        );
        props.insert("details".to_string(), serde_json::json!(sighting.details));

        let feature = Feature {
            bbox: None,
            geometry: Some(geometry),
            id: None,
            properties: Some(props),
            foreign_members: None,
        };

        features.push(feature);
    }

    let feature_collection = FeatureCollection {
        bbox: None,
        features,
        foreign_members: None,
    };

    let json_string = serde_json::to_string_pretty(&feature_collection)?;
    std::fs::write(output_path, json_string)?;

    Ok(())
}

/// Export sightings to KML format (Google Earth)
pub fn export_kml(sightings: &[Sighting], output_path: &str) -> Result<()> {
    let mut file = File::create(output_path)?;

    writeln!(file, r#"<?xml version="1.0" encoding="UTF-8"?>"#)?;
    writeln!(file, r#"<kml xmlns="http://www.opengis.net/kml/2.2">"#)?;
    writeln!(file, r#"  <Document>"#)?;
    writeln!(file, r#"    <name>Wolf Sightings</name>"#)?;
    writeln!(
        file,
        r#"    <description>Exported from Howler</description>"#
    )?;

    for sighting in sightings {
        writeln!(file, r#"    <Placemark>"#)?;
        writeln!(file, r#"      <name>{}</name>"#, sighting.source_id)?;
        writeln!(file, r#"      <description>"#)?;
        writeln!(file, r#"        <![CDATA["#)?;
        let _ = writeln!(
            file,
            r#"          <strong>Species:</strong> {}<br/>"#,
            sighting.species
        );
        if let Some(scientific_name) = &sighting.scientific_name {
            let _ = writeln!(
                file,
                r#"          <strong>Scientific Name:</strong> {}<br/>"#,
                scientific_name
            );
        }
        let _ = writeln!(
            file,
            r#"          <strong>Source:</strong> {}<br/>"#,
            sighting.source
        );
        let _ = writeln!(
            file,
            r#"          <strong>Date:</strong> {}<br/>"#,
            sighting.observed_on.format("%Y-%m-%d %H:%M:%S")
        );
        if let Some(details) = &sighting.details {
            let _ = writeln!(file, r#"          <strong>Details:</strong> {}"#, details);
        }
        writeln!(file, r#"        ]]>"#)?;
        writeln!(file, r#"      </description>"#)?;
        writeln!(file, r#"      <Point>"#)?;
        writeln!(
            file,
            r#"        <coordinates>{},{},0</coordinates>"#,
            sighting.longitude, sighting.latitude
        )?;
        writeln!(file, r#"      </Point>"#)?;
        writeln!(file, r#"    </Placemark>"#)?;
    }

    writeln!(file, r#"  </Document>"#)?;
    writeln!(file, r#"</kml>"#)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::Source;
    use chrono::Utc;
    use tempfile::NamedTempFile;

    fn create_test_sighting(lat: f64, lon: f64, id: i64) -> Sighting {
        Sighting {
            id: Some(id),
            species: "Canis lupus".to_string(),
            scientific_name: Some("Canis lupus".to_string()),
            latitude: lat,
            longitude: lon,
            observed_on: Utc::now(),
            source: Source::GBIF,
            source_id: format!("test_{}", id),
            details: Some("Test sighting".to_string()),
        }
    }

    #[test]
    fn test_export_csv() {
        let sightings = vec![
            create_test_sighting(45.0, -122.0, 1),
            create_test_sighting(46.0, -123.0, 2),
        ];

        let temp_file = NamedTempFile::new().unwrap();
        export_csv(&sightings, temp_file.path().to_str().unwrap()).unwrap();

        let content = std::fs::read_to_string(temp_file.path()).unwrap();
        assert!(content.contains("id,species,scientific_name"));
        assert!(content.contains("Canis lupus"));
    }

    #[test]
    fn test_export_geojson() {
        let sightings = vec![create_test_sighting(45.0, -122.0, 1)];

        let temp_file = NamedTempFile::new().unwrap();
        export_geojson(&sightings, temp_file.path().to_str().unwrap()).unwrap();

        let content = std::fs::read_to_string(temp_file.path()).unwrap();
        assert!(content.contains("FeatureCollection"));
        assert!(content.contains("Point"));
    }

    #[test]
    fn test_export_kml() {
        let sightings = vec![create_test_sighting(45.0, -122.0, 1)];

        let temp_file = NamedTempFile::new().unwrap();
        export_kml(&sightings, temp_file.path().to_str().unwrap()).unwrap();

        let content = std::fs::read_to_string(temp_file.path()).unwrap();
        assert!(content.contains("<kml"));
        assert!(content.contains("<Placemark>"));
        assert!(content.contains("<Point>"));
    }
}
