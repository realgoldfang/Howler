use crate::models::{Sighting, Source};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Filter criteria for sightings
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SightingFilter {
    /// Filter by species name
    pub species: Option<String>,
    /// Filter by data source
    pub source: Option<Source>,
    /// Start date for date range filter
    pub start_date: Option<DateTime<Utc>>,
    /// End date for date range filter
    pub end_date: Option<DateTime<Utc>>,
    /// Minimum latitude
    pub min_latitude: Option<f64>,
    /// Maximum latitude
    pub max_latitude: Option<f64>,
    /// Minimum longitude
    pub min_longitude: Option<f64>,
    /// Maximum longitude
    pub max_longitude: Option<f64>,
    /// Text search in details
    pub search_text: Option<String>,
}

impl SightingFilter {
    /// Create a new empty filter
    pub fn new() -> Self {
        Self::default()
    }

    /// Filter by species
    pub fn with_species(mut self, species: &str) -> Self {
        self.species = Some(species.to_string());
        self
    }

    /// Filter by source
    pub fn with_source(mut self, source: Source) -> Self {
        self.source = Some(source);
        self
    }

    /// Filter by date range
    pub fn with_date_range(mut self, start: DateTime<Utc>, end: DateTime<Utc>) -> Self {
        self.start_date = Some(start);
        self.end_date = Some(end);
        self
    }

    /// Filter by geographic bounds
    pub fn with_bounds(mut self, min_lat: f64, max_lat: f64, min_lon: f64, max_lon: f64) -> Self {
        self.min_latitude = Some(min_lat);
        self.max_latitude = Some(max_lat);
        self.min_longitude = Some(min_lon);
        self.max_longitude = Some(max_lon);
        self
    }

    /// Filter by text search
    pub fn with_search(mut self, text: &str) -> Self {
        self.search_text = Some(text.to_string());
        self
    }

    /// Check if a sighting matches the filter criteria
    pub fn matches(&self, sighting: &Sighting) -> bool {
        // Check species filter
        if let Some(ref species) = self.species {
            if !sighting.species.contains(species) {
                return false;
            }
        }

        // Check source filter
        if let Some(ref source) = self.source {
            if &sighting.source != source {
                return false;
            }
        }

        // Check date range filter
        if let Some(start) = self.start_date {
            if sighting.observed_on < start {
                return false;
            }
        }
        if let Some(end) = self.end_date {
            if sighting.observed_on > end {
                return false;
            }
        }

        // Check latitude bounds
        if let Some(min_lat) = self.min_latitude {
            if sighting.latitude < min_lat {
                return false;
            }
        }
        if let Some(max_lat) = self.max_latitude {
            if sighting.latitude > max_lat {
                return false;
            }
        }

        // Check longitude bounds
        if let Some(min_lon) = self.min_longitude {
            if sighting.longitude < min_lon {
                return false;
            }
        }
        if let Some(max_lon) = self.max_longitude {
            if sighting.longitude > max_lon {
                return false;
            }
        }

        // Check text search
        if let Some(ref search_text) = self.search_text {
            let text_lower = search_text.to_lowercase();
            let matches = sighting.species.to_lowercase().contains(&text_lower)
                || sighting
                    .scientific_name
                    .as_ref()
                    .map(|s| s.to_lowercase().contains(&text_lower))
                    .unwrap_or(false)
                || sighting
                    .details
                    .as_ref()
                    .map(|d| d.to_lowercase().contains(&text_lower))
                    .unwrap_or(false);

            if !matches {
                return false;
            }
        }

        true
    }
}

/// Apply filter to a list of sightings
pub fn filter_sightings(sightings: &[Sighting], filter: &SightingFilter) -> Vec<Sighting> {
    sightings
        .iter()
        .filter(|s| filter.matches(s))
        .cloned()
        .collect()
}

/// Search for sightings by text
pub fn search_sightings(sightings: &[Sighting], query: &str) -> Vec<Sighting> {
    let filter = SightingFilter::new().with_search(query);
    filter_sightings(sightings, &filter)
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Duration;

    fn create_test_sighting(lat: f64, lon: f64, id: i64, hours_ago: i64) -> Sighting {
        Sighting {
            id: Some(id),
            species: "Canis lupus".to_string(),
            scientific_name: Some("Canis lupus".to_string()),
            latitude: lat,
            longitude: lon,
            observed_on: Utc::now() - Duration::hours(hours_ago),
            source: Source::GBIF,
            source_id: format!("test_{}", id),
            details: Some("Yellowstone National Park".to_string()),
        }
    }

    #[test]
    fn test_filter_empty() {
        let sightings = vec![
            create_test_sighting(45.0, -122.0, 1, 0),
            create_test_sighting(46.0, -123.0, 2, 0),
        ];

        let filter = SightingFilter::new();
        let filtered = filter_sightings(&sightings, &filter);
        assert_eq!(filtered.len(), 2);
    }

    #[test]
    fn test_filter_by_species() {
        let sightings = vec![
            create_test_sighting(45.0, -122.0, 1, 0),
            create_test_sighting(46.0, -123.0, 2, 0),
        ];

        let filter = SightingFilter::new().with_species("Canis");
        let filtered = filter_sightings(&sightings, &filter);
        assert_eq!(filtered.len(), 2);
    }

    #[test]
    fn test_filter_by_source() {
        let mut sightings = vec![create_test_sighting(45.0, -122.0, 1, 0)];
        sightings[0].source = Source::INaturalist;

        let filter = SightingFilter::new().with_source(Source::INaturalist);
        let filtered = filter_sightings(&sightings, &filter);
        assert_eq!(filtered.len(), 1);
    }

    #[test]
    fn test_filter_by_bounds() {
        let sightings = vec![
            create_test_sighting(45.0, -122.0, 1, 0),
            create_test_sighting(50.0, -130.0, 2, 0),
        ];

        let filter = SightingFilter::new().with_bounds(44.0, 46.0, -123.0, -121.0);
        let filtered = filter_sightings(&sightings, &filter);
        assert_eq!(filtered.len(), 1);
    }

    #[test]
    fn test_filter_by_search() {
        let sightings = vec![
            create_test_sighting(45.0, -122.0, 1, 0),
            create_test_sighting(46.0, -123.0, 2, 0),
        ];

        let filter = SightingFilter::new().with_search("Yellowstone");
        let filtered = filter_sightings(&sightings, &filter);
        assert_eq!(filtered.len(), 2);
    }

    #[test]
    fn test_search_sightings() {
        let sightings = vec![
            create_test_sighting(45.0, -122.0, 1, 0),
            create_test_sighting(46.0, -123.0, 2, 0),
        ];

        let results = search_sightings(&sightings, "lupus");
        assert_eq!(results.len(), 2);
    }
}
