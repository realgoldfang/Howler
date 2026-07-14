use crate::movebank::MovebankClient;
use crate::Config;

#[test]
fn test_movebank_response_parsing() {
    let json_str = r#"{
  "locations": [
    {
      "location_lat": 48.5,
      "location_long": -113.8,
      "timestamp": "2023-05-15T12:00:00Z",
      "individual_id": 5001,
      "individual_local_identifier": "Wolf_Alpha_001"
    }
  ]
}"#;

    let response: serde_json::Value = serde_json::from_str(json_str).unwrap();
    assert!(response["locations"].is_array());
    assert_eq!(response["locations"].as_array().unwrap().len(), 1);
}

#[test]
fn test_movebank_client_without_credentials() {
    let config = Config {
        movebank_username: None,
        movebank_password: None,
        inaturalist_token: None,
        iucn_token: None,
    };

    let client = MovebankClient::new(&config);
    assert!(client.is_none());
}
