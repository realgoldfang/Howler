use std::env;

pub struct Config {
    pub movebank_username: Option<String>,
    pub movebank_password: Option<String>,
    pub inaturalist_token: Option<String>,
    pub iucn_token: Option<String>,
}

impl Config {
    pub fn from_env() -> Self {
        Self {
            movebank_username: env::var("MOVEBANK_USERNAME").ok(),
            movebank_password: env::var("MOVEBANK_PASSWORD").ok(),
            inaturalist_token: env::var("INATURALIST_TOKEN").ok(),
            iucn_token: env::var("IUCN_TOKEN").ok(),
        }
    }

    pub fn has_movebank_credentials(&self) -> bool {
        self.movebank_username.is_some() && self.movebank_password.is_some()
    }

    pub fn has_inaturalist_token(&self) -> bool {
        self.inaturalist_token.is_some()
    }

    pub fn has_iucn_token(&self) -> bool {
        self.iucn_token.is_some()
    }
}

impl Default for Config {
    fn default() -> Self {
        Self::from_env()
    }
}
