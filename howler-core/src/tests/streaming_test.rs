use crate::models::{Sighting, Source};
use crate::streaming::{Broadcast, StreamingEvent};
use chrono::Utc;

fn test_sighting(id: i64) -> Sighting {
    Sighting {
        id: Some(id),
        species: "Canis lupus".to_string(),
        scientific_name: Some("Canis lupus".to_string()),
        latitude: 45.0 + id as f64,
        longitude: -122.0 - id as f64,
        observed_on: Utc::now(),
        source: Source::GBIF,
        source_id: format!("test_{}", id),
        details: Some("Test sighting".to_string()),
    }
}

#[tokio::test]
async fn test_streaming_broadcast_publish_subscribe() {
    let broadcast = Broadcast::new(32);
    let mut rx = broadcast.subscribe();

    let sighting = test_sighting(1);
    let event = StreamingEvent::SightingCreated(sighting);
    broadcast.publish(event).unwrap();

    let received = rx.recv().await.unwrap();
    match received {
        StreamingEvent::SightingCreated(s) => {
            assert_eq!(s.species, "Canis lupus");
            assert_eq!(s.id, Some(1));
        }
        _ => panic!("Expected SightingCreated"),
    }
}

#[tokio::test]
async fn test_streaming_broadcast_multiple_events() {
    let broadcast = Broadcast::new(32);
    let mut rx = broadcast.subscribe();

    for i in 1..=3 {
        let sighting = test_sighting(i);
        broadcast
            .publish(StreamingEvent::SightingCreated(sighting))
            .unwrap();
    }

    for i in 1..=3 {
        let received = rx.recv().await.unwrap();
        match received {
            StreamingEvent::SightingCreated(s) => assert_eq!(s.id, Some(i)),
            _ => panic!("Expected SightingCreated"),
        }
    }
}

#[tokio::test]
async fn test_streaming_broadcast_analysis_complete() {
    let broadcast = Broadcast::new(32);
    let mut rx = broadcast.subscribe();

    let event = StreamingEvent::AnalysisComplete("Cluster analysis done".to_string());
    broadcast.publish(event).unwrap();

    let received = rx.recv().await.unwrap();
    match received {
        StreamingEvent::AnalysisComplete(msg) => {
            assert_eq!(msg, "Cluster analysis done");
        }
        _ => panic!("Expected AnalysisComplete"),
    }
}

#[tokio::test]
async fn test_streaming_broadcast_multiple_subscribers() {
    let broadcast = Broadcast::new(32);
    let mut rx1 = broadcast.subscribe();
    let mut rx2 = broadcast.subscribe();

    let sighting = test_sighting(1);
    let event = StreamingEvent::SightingUpdated(sighting);
    broadcast.publish(event).unwrap();

    let r1 = rx1.recv().await.unwrap();
    let r2 = rx2.recv().await.unwrap();

    match (&r1, &r2) {
        (
            StreamingEvent::SightingUpdated(s1),
            StreamingEvent::SightingUpdated(s2),
        ) => {
            assert_eq!(s1.id, s2.id);
            assert_eq!(s1.species, s2.species);
        }
        _ => panic!("Expected SightingUpdated for both"),
    }
}

#[tokio::test]
async fn test_streaming_event_serialization() {
    let sighting = test_sighting(42);
    let event = StreamingEvent::SightingCreated(sighting);

    let json = serde_json::to_string(&event).unwrap();
    assert!(json.contains("SightingCreated"));
    assert!(json.contains("Canis lupus"));

    let deserialized: StreamingEvent = serde_json::from_str(&json).unwrap();
    match deserialized {
        StreamingEvent::SightingCreated(s) => {
            assert_eq!(s.id, Some(42));
            assert_eq!(s.species, "Canis lupus");
        }
        _ => panic!("Expected SightingCreated"),
    }
}

#[tokio::test]
async fn test_streaming_broadcast_event_types_roundtrip() {
    let broadcast = Broadcast::new(32);
    let mut rx = broadcast.subscribe();

    let sighting = test_sighting(10);
    let created = StreamingEvent::SightingCreated(sighting.clone());
    let updated = StreamingEvent::SightingUpdated(sighting.clone());
    let analysis = StreamingEvent::AnalysisComplete("done".to_string());

    broadcast.publish(created).unwrap();
    broadcast.publish(updated).unwrap();
    broadcast.publish(analysis).unwrap();

    let e1 = rx.recv().await.unwrap();
    let e2 = rx.recv().await.unwrap();
    let e3 = rx.recv().await.unwrap();

    assert!(matches!(e1, StreamingEvent::SightingCreated(_)));
    assert!(matches!(e2, StreamingEvent::SightingUpdated(_)));
    assert!(matches!(e3, StreamingEvent::AnalysisComplete(_)));

    let json1 = serde_json::to_string(&e1).unwrap();
    let d1: StreamingEvent = serde_json::from_str(&json1).unwrap();
    assert!(matches!(d1, StreamingEvent::SightingCreated(_)));
}
