use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash, uniffi::Enum)]
pub enum Source {
    GBIF,
    Movebank,
    INaturalist,
    IUCN,
}

impl std::fmt::Display for Source {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Source::GBIF => write!(f, "GBIF"),
            Source::Movebank => write!(f, "Movebank"),
            Source::INaturalist => write!(f, "iNaturalist"),
            Source::IUCN => write!(f, "IUCN"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Sighting {
    pub id: Option<i64>,
    pub species: String,
    pub scientific_name: Option<String>,
    pub latitude: f64,
    pub longitude: f64,
    pub observed_on: DateTime<Utc>,
    pub source: Source,
    pub source_id: String,
    pub details: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpeciesStatus {
    pub id: Option<i64>,
    pub scientific_name: String,
    pub common_name: Option<String>,
    pub red_list_category: Option<String>,
    pub population_trend: Option<String>,
    pub threats: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, uniffi::Record)]
pub struct MobileSighting {
    pub id: Option<i64>,
    pub species: String,
    pub scientific_name: Option<String>,
    pub latitude: f64,
    pub longitude: f64,
    pub observed_on: i64,
    pub source: Source,
    pub source_id: String,
    pub details: Option<String>,
}

impl From<&Sighting> for MobileSighting {
    fn from(s: &Sighting) -> Self {
        MobileSighting {
            id: s.id,
            species: s.species.clone(),
            scientific_name: s.scientific_name.clone(),
            latitude: s.latitude,
            longitude: s.longitude,
            observed_on: s.observed_on.timestamp(),
            source: s.source.clone(),
            source_id: s.source_id.clone(),
            details: s.details.clone(),
        }
    }
}

impl From<&MobileSighting> for Sighting {
    fn from(s: &MobileSighting) -> Self {
        Sighting {
            id: s.id,
            species: s.species.clone(),
            scientific_name: s.scientific_name.clone(),
            latitude: s.latitude,
            longitude: s.longitude,
            observed_on: DateTime::from_timestamp(s.observed_on, 0).unwrap_or_default(),
            source: s.source.clone(),
            source_id: s.source_id.clone(),
            details: s.details.clone(),
        }
    }
}
