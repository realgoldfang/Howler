#[test]
fn test_iucn_response_parsing() {
    let json_str = r#"{
  "result": [
    {
      "taxonid": 3761,
      "scientific_name": "Canis lupus",
      "common_name": "Grey Wolf",
      "main_common_name": "Gray Wolf",
      "category": "LC",
      "population_trend": "Stable"
    }
  ]
}"#;

    let response: serde_json::Value = serde_json::from_str(json_str).unwrap();
    assert!(response["result"].is_array());
    assert_eq!(response["result"].as_array().unwrap().len(), 1);
}
