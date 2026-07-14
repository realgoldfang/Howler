use crate::config::Config;
use crate::db::Database;
use crate::models::{Sighting, Source};
use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::Deserialize;

const MOVEBANK_API_BASE: &str = "https://www.movebank.org/movebank-service/djson";

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct MovebankResponse {
    #[serde(rename = "individuals")]
    individuals: Option<Vec<MovebankIndividual>>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct MovebankIndividual {
    #[serde(rename = "individual_id")]
    individual_id: Option<i64>,
    #[serde(rename = "individual_local_identifier")]
    individual_local_identifier: Option<String>,
    #[serde(rename = "taxon_canonical_name")]
    taxon_canonical_name: Option<String>,
}

#[derive(Debug, Deserialize)]
struct MovebankLocationData {
    #[serde(rename = "locations")]
    locations: Option<Vec<MovebankLocation>>,
}

#[derive(Debug, Deserialize)]
struct MovebankLocation {
    #[serde(rename = "location_lat")]
    location_lat: Option<f64>,
    #[serde(rename = "location_long")]
    location_long: Option<f64>,
    #[serde(rename = "timestamp")]
    timestamp: Option<String>,
    #[serde(rename = "individual_id")]
    individual_id: Option<i64>,
    #[serde(rename = "individual_local_identifier")]
    individual_local_identifier: Option<String>,
}

pub struct MovebankClient {
    client: reqwest::Client,
    username: String,
    password: String,
}

impl MovebankClient {
    pub fn new(config: &Config) -> Option<Self> {
        if !config.has_movebank_credentials() {
            return None;
        }

        Some(Self {
            client: reqwest::Client::new(),
            username: config.movebank_username.clone().unwrap(),
            password: config.movebank_password.clone().unwrap(),
        })
    }

    pub async fn fetch_wolf_tracks(&self, limit: u32) -> Result<Vec<Sighting>> {
        // First, get a list of studies with wolf data
        let studies_url = format!(
            "{}?study_id=2928297&individual_local_identifier=*",
            MOVEBANK_API_BASE
        );

        let response = self
            .client
            .get(&studies_url)
            .basic_auth(&self.username, Some(&self.password))
            .header("User-Agent", "Howler/0.1.0")
            .send()
            .await
            .context("Failed to send Movebank request")?;

        if !response.status().is_success() {
            anyhow::bail!("Movebank API returned error: {}", response.status());
        }

        // For now, we'll use a simplified approach - fetch location data directly
        // In a full implementation, you'd first list studies, then individuals, then locations
        let locations_url = format!("{}?study_id=2928297&sensor_type_id=653", MOVEBANK_API_BASE);

        let loc_response = self
            .client
            .get(&locations_url)
            .basic_auth(&self.username, Some(&self.password))
            .header("User-Agent", "Howler/0.1.0")
            .send()
            .await
            .context("Failed to fetch Movebank locations")?;

        if !loc_response.status().is_success() {
            // If study-specific request fails, return empty but don't crash
            return Ok(vec![]);
        }

        let location_data: MovebankLocationData = loc_response
            .json()
            .await
            .context("Failed to parse Movebank location data")?;

        let sightings = location_data
            .locations
            .unwrap_or_default()
            .into_iter()
            .filter_map(|loc| self.location_to_sighting(loc))
            .take(limit as usize)
            .collect();

        Ok(sightings)
    }

    fn location_to_sighting(&self, loc: MovebankLocation) -> Option<Sighting> {
        let latitude = loc.location_lat?;
        let longitude = loc.location_long?;

        let observed_on = if let Some(ts_str) = loc.timestamp {
            DateTime::parse_from_rfc3339(&ts_str)
                .or_else(|_| DateTime::parse_from_rfc2822(&ts_str))
                .map(|dt| dt.with_timezone(&Utc))
                .ok()
        } else {
            None
        }
        .unwrap_or_else(Utc::now);

        let individual_id = loc.individual_local_identifier.unwrap_or_else(|| {
            loc.individual_id
                .map(|id| id.to_string())
                .unwrap_or_default()
        });

        Some(Sighting {
            id: None,
            species: "Canis lupus".to_string(),
            scientific_name: Some("Canis lupus".to_string()),
            latitude,
            longitude,
            observed_on,
            source: Source::Movebank,
            source_id: format!("movebank_{}", individual_id),
            details: Some(format!("Individual: {}", individual_id)),
        })
    }
}

pub async fn fetch_and_cache_movebank(db: &Database, config: &Config, limit: u32) -> Result<usize> {
    let client = match MovebankClient::new(config) {
        Some(c) => c,
        None => {
            eprintln!("Movebank credentials not found, skipping Movebank data");
            return Ok(0);
        }
    };

    let sightings = client.fetch_wolf_tracks(limit).await?;

    let mut count = 0;
    for sighting in sightings {
        db.insert_sighting(&sighting)
            .context("Failed to insert sighting into database")?;
        count += 1;
    }

    Ok(count)
}
