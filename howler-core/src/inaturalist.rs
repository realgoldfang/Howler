use crate::config::Config;
use crate::db::Database;
use crate::models::{Sighting, Source};
use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::Deserialize;

const INATURALIST_API_BASE: &str = "https://api.inaturalist.org/v1/observations";

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct INatResponse {
    results: Vec<INatObservation>,
    total_results: Option<i32>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct INatObservation {
    id: Option<i64>,
    species_guess: Option<String>,
    taxon: Option<INatTaxon>,
    location: Option<String>,
    observed_on: Option<String>,
    observed_on_string: Option<String>,
    time_observed_at: Option<String>,
    photos: Option<Vec<INatPhoto>>,
    place_guess: Option<String>,
}

#[derive(Debug, Deserialize)]
struct INatTaxon {
    name: Option<String>,
    preferred_common_name: Option<String>,
}

#[derive(Debug, Deserialize)]
struct INatPhoto {
    url: Option<String>,
}

pub struct INaturalistClient {
    client: reqwest::Client,
    token: Option<String>,
}

impl INaturalistClient {
    pub fn new(config: &Config) -> Self {
        Self {
            client: reqwest::Client::new(),
            token: config.inaturalist_token.clone(),
        }
    }

    pub async fn fetch_wolf_sightings(&self, limit: u32) -> Result<Vec<Sighting>> {
        let url = format!(
            "{}?taxon_name=Canis%20lupus&geo=true&has_photos=true&per_page={}",
            INATURALIST_API_BASE, limit
        );

        let mut request = self.client.get(&url).header("User-Agent", "Howler/0.1.0");

        if let Some(token) = &self.token {
            request = request.header("Authorization", format!("Bearer {}", token));
        }

        let response = request
            .send()
            .await
            .context("Failed to send iNaturalist request")?;

        if !response.status().is_success() {
            anyhow::bail!("iNaturalist API returned error: {}", response.status());
        }

        let inat_response: INatResponse = response
            .json()
            .await
            .context("Failed to parse iNaturalist response")?;

        let sightings = inat_response
            .results
            .into_iter()
            .filter_map(|obs| self.observation_to_sighting(obs))
            .collect();

        Ok(sightings)
    }

    fn observation_to_sighting(&self, obs: INatObservation) -> Option<Sighting> {
        let location_str = obs.location?;
        let coords: Vec<&str> = location_str.split(',').collect();
        if coords.len() != 2 {
            return None;
        }

        let latitude = coords[0].trim().parse().ok()?;
        let longitude = coords[1].trim().parse().ok()?;

        let observed_on = obs
            .time_observed_at
            .or(obs.observed_on)
            .and_then(|ts_str| {
                DateTime::parse_from_rfc3339(&ts_str)
                    .or_else(|_| DateTime::parse_from_rfc2822(&ts_str))
                    .map(|dt| dt.with_timezone(&Utc))
                    .ok()
            })
            .unwrap_or_else(Utc::now);

        let obs_id = obs.id.map(|id| id.to_string()).unwrap_or_default();

        let scientific_name = obs.taxon.as_ref().and_then(|t| t.name.clone());
        let species = obs
            .taxon
            .as_ref()
            .and_then(|t| t.preferred_common_name.clone())
            .or(obs.species_guess)
            .unwrap_or_else(|| "Canis lupus".to_string());

        let photo_urls = obs.photos.as_ref().map(|photos| {
            photos
                .iter()
                .filter_map(|p| p.url.clone())
                .collect::<Vec<_>>()
                .join(", ")
        });

        let details = Some(format!(
            "Location: {} | Photos: {}",
            obs.place_guess.unwrap_or_else(|| "Unknown".to_string()),
            photo_urls.unwrap_or_else(|| "None".to_string())
        ));

        Some(Sighting {
            id: None,
            species,
            scientific_name,
            latitude,
            longitude,
            observed_on,
            source: Source::INaturalist,
            source_id: format!("inaturalist_{}", obs_id),
            details,
        })
    }
}

pub async fn fetch_and_cache_inaturalist(
    db: &Database,
    config: &Config,
    limit: u32,
) -> Result<usize> {
    if !config.has_inaturalist_token() {
        eprintln!("iNaturalist: skipping (no token configured). To enable, set INATURALIST_TOKEN.");
        return Ok(0);
    }

    let client = INaturalistClient::new(config);
    let sightings = client.fetch_wolf_sightings(limit).await?;

    let mut count = 0;
    for sighting in sightings {
        db.insert_sighting(&sighting)
            .context("Failed to insert sighting into database")?;
        count += 1;
    }

    Ok(count)
}
