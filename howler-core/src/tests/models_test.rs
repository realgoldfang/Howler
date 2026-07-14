use crate::models::{Sighting, Source, SpeciesStatus};
use chrono::Utc;

#[test]
fn test_source_display() {
    assert_eq!(Source::GBIF.to_string(), "GBIF");
    assert_eq!(Source::Movebank.to_string(), "Movebank");
    assert_eq!(Source::INaturalist.to_string(), "iNaturalist");
    assert_eq!(Source::IUCN.to_string(), "IUCN");
}

#[test]
fn test_sighting_creation() {
    let sighting = Sighting {
        id: None,
        species: "Canis lupus".to_string(),
        scientific_name: Some("Canis lupus".to_string()),
        latitude: 45.5,
        longitude: -122.5,
        observed_on: Utc::now(),
        source: Source::GBIF,
        source_id: "12345".to_string(),
        details: Some("Test sighting".to_string()),
    };

    assert_eq!(sighting.species, "Canis lupus");
    assert_eq!(sighting.latitude, 45.5);
    assert_eq!(sighting.longitude, -122.5);
    assert_eq!(sighting.source, Source::GBIF);
}

#[test]
fn test_species_status_creation() {
    let status = SpeciesStatus {
        id: None,
        scientific_name: "Canis lupus".to_string(),
        common_name: Some("Gray Wolf".to_string()),
        red_list_category: Some("LC".to_string()),
        population_trend: Some("Stable".to_string()),
        threats: Some("Habitat loss".to_string()),
    };

    assert_eq!(status.scientific_name, "Canis lupus");
    assert_eq!(status.common_name, Some("Gray Wolf".to_string()));
    assert_eq!(status.red_list_category, Some("LC".to_string()));
}
