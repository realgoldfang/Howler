use crate::Config;
use std::env;

#[test]
fn test_config_from_env() {
    env::set_var("MOVEBANK_USERNAME", "test_user");
    env::set_var("MOVEBANK_PASSWORD", "test_pass");
    env::set_var("INATURALIST_TOKEN", "test_inat_token");
    env::set_var("IUCN_TOKEN", "test_iucn_token");

    let config = Config::from_env();

    assert_eq!(config.movebank_username, Some("test_user".to_string()));
    assert_eq!(config.movebank_password, Some("test_pass".to_string()));
    assert_eq!(
        config.inaturalist_token,
        Some("test_inat_token".to_string())
    );
    assert_eq!(config.iucn_token, Some("test_iucn_token".to_string()));

    // Clean up
    env::remove_var("MOVEBANK_USERNAME");
    env::remove_var("MOVEBANK_PASSWORD");
    env::remove_var("INATURALIST_TOKEN");
    env::remove_var("IUCN_TOKEN");
}

#[test]
fn test_config_empty_env() {
    env::remove_var("MOVEBANK_USERNAME");
    env::remove_var("MOVEBANK_PASSWORD");
    env::remove_var("INATURALIST_TOKEN");
    env::remove_var("IUCN_TOKEN");

    let config = Config::from_env();

    assert_eq!(config.movebank_username, None);
    assert_eq!(config.movebank_password, None);
    assert_eq!(config.inaturalist_token, None);
    assert_eq!(config.iucn_token, None);
}
