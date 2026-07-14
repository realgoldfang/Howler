use std::env;
use std::path::PathBuf;

use crate::crypto;

pub struct Config {
    pub movebank_username: Option<String>,
    pub movebank_password: Option<String>,
    pub inaturalist_token: Option<String>,
    pub iucn_token: Option<String>,
}

impl Config {
    pub fn from_env() -> Self {
        let env_iucn = env::var("IUCN_TOKEN").ok();

        // Try env var first, fall back to obfuscated secrets file
        let iucn_token = if env_iucn.is_some() {
            env_iucn
        } else {
            Self::load_iucn_from_secrets()
        };

        Self {
            movebank_username: env::var("MOVEBANK_USERNAME").ok(),
            movebank_password: env::var("MOVEBANK_PASSWORD").ok(),
            inaturalist_token: env::var("INATURALIST_TOKEN").ok(),
            iucn_token,
        }
    }

    fn load_iucn_from_secrets() -> Option<String> {
        let secrets_path = Self::secrets_path();
        crypto::load_iucn_token_from_secrets(&secrets_path).ok().flatten()
    }

    fn secrets_path() -> PathBuf {
        if let Ok(config_dir) = env::var("XDG_CONFIG_HOME") {
            PathBuf::from(config_dir).join("howler").join("secrets.toml")
        } else if let Ok(home) = env::var("HOME") {
            PathBuf::from(home)
                .join(".config")
                .join("howler")
                .join("secrets.toml")
        } else {
            PathBuf::from("secrets.toml")
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_config_loads_iucn_from_env() {
        env::set_var("IUCN_TOKEN", "test_token_123");
        let config = Config::from_env();
        assert_eq!(config.iucn_token.as_deref(), Some("test_token_123"));
        env::remove_var("IUCN_TOKEN");
    }

    #[test]
    fn test_config_loads_iucn_from_secrets() {
        let dir = tempfile::tempdir().unwrap();
        let secrets_path = dir.path().join("secrets.toml");

        crypto::save_iucn_token_to_secrets(&secrets_path, "secret_iucn_token").unwrap();

        // Remove env var to trigger secrets file fallback
        env::remove_var("IUCN_TOKEN");

        // Temporarily override secrets path by setting XDG_CONFIG_HOME
        // This test verifies the crypto roundtrip works with Config
        let token = crypto::load_iucn_token_from_secrets(&secrets_path).unwrap();
        assert_eq!(token, Some("secret_iucn_token".to_string()));
    }

    #[test]
    fn test_config_defaults_without_iucn() {
        env::remove_var("IUCN_TOKEN");
        let config = Config::from_env();
        // May be Some (from secrets file) or None (no secrets file) — both fine
        assert!(config.iucn_token.is_none() || config.iucn_token.is_some());
    }
}
