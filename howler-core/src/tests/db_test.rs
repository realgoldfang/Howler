use crate::models::SpeciesStatus;
use crate::{Database, Sighting, Source};
use chrono::Utc;
use tempfile::NamedTempFile;

#[test]
fn test_database_creation() {
    let temp_file = NamedTempFile::new().unwrap();
    let db = Database::new(temp_file.path().to_str().unwrap()).unwrap();
    // Test that database was created successfully by inserting a sighting
    let sighting = Sighting {
        id: None,
        species: "Test".to_string(),
        scientific_name: None,
        latitude: 0.0,
        longitude: 0.0,
        observed_on: Utc::now(),
        source: Source::GBIF,
        source_id: "test".to_string(),
        details: None,
    };
    assert!(db.insert_sighting(&sighting).is_ok());
}

#[test]
fn test_insert_and_get_sighting() {
    let temp_file = NamedTempFile::new().unwrap();
    let db = Database::new(temp_file.path().to_str().unwrap()).unwrap();

    let sighting = Sighting {
        id: None,
        species: "Canis lupus".to_string(),
        scientific_name: Some("Canis lupus".to_string()),
        latitude: 45.5,
        longitude: -122.5,
        observed_on: Utc::now(),
        source: Source::GBIF,
        source_id: "test_123".to_string(),
        details: Some("Test sighting".to_string()),
    };

    let id = db.insert_sighting(&sighting).unwrap();
    assert!(id > 0);

    let sightings = db.get_all_sightings().unwrap();
    assert_eq!(sightings.len(), 1);
    assert_eq!(sightings[0].species, "Canis lupus");
    assert_eq!(sightings[0].source_id, "test_123");
}

#[test]
fn test_insert_duplicate_sighting() {
    let temp_file = NamedTempFile::new().unwrap();
    let db = Database::new(temp_file.path().to_str().unwrap()).unwrap();

    let sighting = Sighting {
        id: None,
        species: "Canis lupus".to_string(),
        scientific_name: Some("Canis lupus".to_string()),
        latitude: 45.5,
        longitude: -122.5,
        observed_on: Utc::now(),
        source: Source::GBIF,
        source_id: "test_123".to_string(),
        details: Some("Test sighting".to_string()),
    };

    db.insert_sighting(&sighting).unwrap();
    db.insert_sighting(&sighting).unwrap(); // Should replace due to UNIQUE constraint

    let sightings = db.get_all_sightings().unwrap();
    assert_eq!(sightings.len(), 1);
}

#[test]
fn test_insert_and_get_species_status() {
    let temp_file = NamedTempFile::new().unwrap();
    let db = Database::new(temp_file.path().to_str().unwrap()).unwrap();

    let status = SpeciesStatus {
        id: None,
        scientific_name: "Canis lupus".to_string(),
        common_name: Some("Gray Wolf".to_string()),
        red_list_category: Some("LC".to_string()),
        population_trend: Some("Stable".to_string()),
        threats: Some("Habitat loss".to_string()),
    };

    let id = db.insert_species_status(&status).unwrap();
    assert!(id > 0);

    let retrieved = db.get_species_status("Canis lupus").unwrap();
    assert!(retrieved.is_some());
    let retrieved = retrieved.unwrap();
    assert_eq!(retrieved.scientific_name, "Canis lupus");
    assert_eq!(retrieved.common_name, Some("Gray Wolf".to_string()));
}

#[test]
fn test_get_nonexistent_species_status() {
    let temp_file = NamedTempFile::new().unwrap();
    let db = Database::new(temp_file.path().to_str().unwrap()).unwrap();

    let retrieved = db.get_species_status("Nonexistent species").unwrap();
    assert!(retrieved.is_none());
}
