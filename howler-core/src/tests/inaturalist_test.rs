#[test]
fn test_inaturalist_response_parsing() {
    let json_str = r#"{
  "results": [
    {
      "id": 1001,
      "species_guess": "Canis lupus",
      "taxon": {
        "name": "Canis lupus",
        "preferred_common_name": "Gray Wolf"
      },
      "location": "44.5,-110.5",
      "observed_on": "2023-03-10T08:15:00+00:00",
      "time_observed_at": "2023-03-10T08:15:00Z",
      "photos": [
        {
          "url": "https://example.com/wolf1.jpg"
        }
      ],
      "place_guess": "Yellowstone National Park"
    }
  ],
  "total_results": 1
}"#;

    let response: serde_json::Value = serde_json::from_str(json_str).unwrap();
    assert!(response["results"].is_array());
    assert_eq!(response["results"].as_array().unwrap().len(), 1);
}
