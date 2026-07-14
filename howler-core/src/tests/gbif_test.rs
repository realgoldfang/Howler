#[test]
fn test_gbif_response_parsing() {
    let json_str = r#"{
  "results": [
    {
      "species": "Canis lupus",
      "scientificName": "Canis lupus",
      "decimalLatitude": 45.5,
      "decimalLongitude": -122.5,
      "eventDate": "2023-01-15T10:30:00Z",
      "gbifId": 12345,
      "occurrenceId": "gbif:12345",
      "verbatimLocality": "Yellowstone National Park"
    }
  ]
}"#;

    let response: serde_json::Value = serde_json::from_str(json_str).unwrap();
    assert!(response["results"].is_array());
    assert_eq!(response["results"].as_array().unwrap().len(), 1);
}
