use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
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
