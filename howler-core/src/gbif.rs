use crate::db::Database;
use crate::models::{Sighting, Source};
use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::Deserialize;

const GBIF_API_BASE: &str = "https://api.gbif.org/v1/occurrence/search";

#[derive(Debug, Deserialize)]
struct GBIFResponse {
    results: Vec<GBIFOccurrence>,
}

#[derive(Debug, Deserialize)]
struct GBIFOccurrence {
    species: Option<String>,
    scientific_name: Option<String>,
    decimal_latitude: Option<f64>,
    decimal_longitude: Option<f64>,
    event_date: Option<String>,
    gbif_id: Option<i64>,
    occurrence_id: Option<String>,
    #[serde(rename = "verbatimLocality")]
    verbatim_locality: Option<String>,
}

pub struct GBIFClient {
    client: reqwest::Client,
}

impl GBIFClient {
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::new(),
        }
    }

    pub async fn fetch_wolf_sightings(&self, limit: u32) -> Result<Vec<Sighting>> {
        let url = format!(
            "{}?speciesKey=5219408&hasCoordinate=true&limit={}",
            GBIF_API_BASE, limit
        );

        let response = self
            .client
            .get(&url)
            .header("User-Agent", "Howler/0.1.0 (wolf tracking application)")
            .send()
            .await
            .context("Failed to send GBIF request")?;

        if !response.status().is_success() {
            anyhow::bail!("GBIF API returned error: {}", response.status());
        }

        let gbif_response: GBIFResponse = response
            .json()
            .await
            .context("Failed to parse GBIF response")?;

        let sightings = gbif_response
            .results
            .into_iter()
            .filter_map(|occ| self.occurrence_to_sighting(occ))
            .collect();

        Ok(sightings)
    }

    fn occurrence_to_sighting(&self, occ: GBIFOccurrence) -> Option<Sighting> {
        let latitude = occ.decimal_latitude?;
        let longitude = occ.decimal_longitude?;

        let observed_on = if let Some(date_str) = occ.event_date {
            DateTime::parse_from_rfc3339(&date_str)
                .or_else(|_| DateTime::parse_from_rfc2822(&date_str))
                .map(|dt| dt.with_timezone(&Utc))
                .ok()
        } else {
            None
        }
        .unwrap_or_else(Utc::now);

        let source_id = occ
            .occurrence_id
            .unwrap_or_else(|| occ.gbif_id.map(|id| id.to_string()).unwrap_or_default());

        Some(Sighting {
            id: None,
            species: occ.species.unwrap_or_else(|| "Canis lupus".to_string()),
            scientific_name: occ.scientific_name,
            latitude,
            longitude,
            observed_on,
            source: Source::GBIF,
            source_id,
            details: occ.verbatim_locality,
        })
    }
}

impl Default for GBIFClient {
    fn default() -> Self {
        Self::new()
    }
}

pub async fn fetch_and_cache_gbif(db: &Database, limit: u32) -> Result<usize> {
    let client = GBIFClient::new();
    let sightings = client.fetch_wolf_sightings(limit).await?;

    let mut count = 0;
    for sighting in sightings {
        db.insert_sighting(&sighting)
            .context("Failed to insert sighting into database")?;
        count += 1;
    }

    Ok(count)
}
