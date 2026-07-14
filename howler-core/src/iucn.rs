use crate::config::Config;
use crate::db::Database;
use crate::models::SpeciesStatus;
use anyhow::{Context, Result};
use serde::Deserialize;

const IUCN_API_BASE: &str = "https://apiv3.iucnredlist.org/api/v3";

#[derive(Debug, Deserialize)]
struct IUCNSpeciesResponse {
    result: Vec<IUCNSpecies>,
}

#[derive(Debug, Deserialize)]
struct IUCNSpecies {
    taxonid: Option<i64>,
    scientific_name: Option<String>,
    common_name: Option<String>,
    category: Option<String>,
    population_trend: Option<String>,
    main_common_name: Option<String>,
}

#[derive(Debug, Deserialize)]
struct IUCNThreatsResponse {
    result: Vec<IUCNThreat>,
}

#[derive(Debug, Deserialize)]
struct IUCNThreat {
    title: Option<String>,
    code: Option<String>,
}

pub struct IUCNClient {
    client: reqwest::Client,
    token: Option<String>,
}

impl IUCNClient {
    pub fn new(config: &Config) -> Self {
        Self {
            client: reqwest::Client::new(),
            token: config.iucn_token.clone(),
        }
    }

    pub async fn fetch_species_status(
        &self,
        scientific_name: &str,
    ) -> Result<Option<SpeciesStatus>> {
        let token = match &self.token {
            Some(t) => t,
            None => {
                eprintln!("IUCN token not found, skipping IUCN data");
                return Ok(None);
            }
        };

        // First, search for the species by name
        let search_url = format!(
            "{}/species/search?token={}&name={}",
            IUCN_API_BASE, token, scientific_name
        );

        let response = self
            .client
            .get(&search_url)
            .header("User-Agent", "Howler/0.1.0")
            .send()
            .await
            .context("Failed to send IUCN search request")?;

        if !response.status().is_success() {
            eprintln!("IUCN API returned error: {}", response.status());
            return Ok(None);
        }

        let species_response: IUCNSpeciesResponse = response
            .json()
            .await
            .context("Failed to parse IUCN species response")?;

        let species = match species_response.result.first() {
            Some(s) => s,
            None => return Ok(None),
        };

        // Fetch threats for this species
        let taxon_id = species.taxonid.unwrap_or(0);
        let threats_url = format!(
            "{}/threats/species/id/{}?token={}",
            IUCN_API_BASE, taxon_id, token
        );

        let threats_response = self
            .client
            .get(&threats_url)
            .header("User-Agent", "Howler/0.1.0")
            .send()
            .await;

        let threats = if let Ok(resp) = threats_response {
            if resp.status().is_success() {
                if let Ok(threats_data) = resp.json::<IUCNThreatsResponse>().await {
                    Some(
                        threats_data
                            .result
                            .iter()
                            .filter_map(|t| {
                                let title = t.title.as_ref()?;
                                let code = t.code.as_ref()?;
                                Some(format!("{} ({})", title, code))
                            })
                            .collect::<Vec<_>>()
                            .join("; "),
                    )
                } else {
                    None
                }
            } else {
                None
            }
        } else {
            None
        };

        Ok(Some(SpeciesStatus {
            id: None,
            scientific_name: species
                .scientific_name
                .clone()
                .unwrap_or_else(|| scientific_name.to_string()),
            common_name: species
                .main_common_name
                .clone()
                .or(species.common_name.clone()),
            red_list_category: species.category.clone(),
            population_trend: species.population_trend.clone(),
            threats,
        }))
    }

    pub async fn fetch_wolf_status(&self) -> Result<Option<SpeciesStatus>> {
        self.fetch_species_status("Canis lupus").await
    }
}

pub async fn fetch_and_cache_iucn(db: &Database, config: &Config) -> Result<usize> {
    let client = IUCNClient::new(config);

    if !config.has_iucn_token() {
        eprintln!("IUCN token not found, skipping IUCN data");
        return Ok(0);
    }

    let status = client.fetch_wolf_status().await?;

    if let Some(status) = status {
        db.insert_species_status(&status)
            .context("Failed to insert species status into database")?;
        Ok(1)
    } else {
        Ok(0)
    }
}
