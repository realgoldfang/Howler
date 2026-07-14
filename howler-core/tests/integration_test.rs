use chrono::Utc;
use howler_core::models::SpeciesStatus;
use howler_core::{Database, Sighting, Source};
use tempfile::NamedTempFile;

#[test]
fn test_integration_database_workflow() {
    let temp_file = NamedTempFile::new().unwrap();
    let db = Database::new(temp_file.path().to_str().unwrap()).unwrap();

    // Insert multiple sightings
    let sighting1 = Sighting {
        id: None,
        species: "Canis lupus".to_string(),
        scientific_name: Some("Canis lupus".to_string()),
        latitude: 45.5,
        longitude: -122.5,
        observed_on: Utc::now(),
        source: Source::GBIF,
        source_id: "gbif_123".to_string(),
        details: Some("Yellowstone".to_string()),
    };

    let sighting2 = Sighting {
        id: None,
        species: "Canis lupus".to_string(),
        scientific_name: Some("Canis lupus".to_string()),
        latitude: 46.8,
        longitude: -121.7,
        observed_on: Utc::now(),
        source: Source::INaturalist,
        source_id: "inat_456".to_string(),
        details: Some("Mount Rainier".to_string()),
    };

    db.insert_sighting(&sighting1).unwrap();
    db.insert_sighting(&sighting2).unwrap();

    // Retrieve and verify
    let sightings = db.get_all_sightings().unwrap();
    assert_eq!(sightings.len(), 2);

    // Insert species status
    let status = SpeciesStatus {
        id: None,
        scientific_name: "Canis lupus".to_string(),
        common_name: Some("Gray Wolf".to_string()),
        red_list_category: Some("LC".to_string()),
        population_trend: Some("Stable".to_string()),
        threats: Some("Habitat loss".to_string()),
    };

    db.insert_species_status(&status).unwrap();

    let retrieved_status = db.get_species_status("Canis lupus").unwrap();
    assert!(retrieved_status.is_some());
    assert_eq!(
        retrieved_status.unwrap().common_name,
        Some("Gray Wolf".to_string())
    );
}

#[test]
fn test_integration_duplicate_handling() {
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
        source_id: "duplicate_test".to_string(),
        details: Some("Test".to_string()),
    };

    // Insert same sighting twice
    db.insert_sighting(&sighting).unwrap();
    db.insert_sighting(&sighting).unwrap();

    let sightings = db.get_all_sightings().unwrap();
    assert_eq!(sightings.len(), 1); // Should only have one due to UNIQUE constraint
}
