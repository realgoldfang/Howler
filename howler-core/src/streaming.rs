use crate::models::Sighting;
use serde::{Deserialize, Serialize};
use tokio::sync::broadcast;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum StreamingEvent {
    SightingCreated(Sighting),
    SightingUpdated(Sighting),
    AnalysisComplete(String),
}

#[derive(Clone)]
pub struct Broadcast {
    sender: broadcast::Sender<StreamingEvent>,
}

impl Broadcast {
    pub fn new(capacity: usize) -> Self {
        let (sender, _) = broadcast::channel(capacity);
        Broadcast { sender }
    }

    pub fn subscribe(&self) -> broadcast::Receiver<StreamingEvent> {
        self.sender.subscribe()
    }

    pub fn publish(
        &self,
        event: StreamingEvent,
    ) -> Result<(), Box<broadcast::error::SendError<StreamingEvent>>> {
        self.sender.send(event).map(|_| ()).map_err(Box::new)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::Source;
    use chrono::Utc;

    fn test_sighting() -> Sighting {
        Sighting {
            id: Some(1),
            species: "Canis lupus".to_string(),
            scientific_name: Some("Canis lupus".to_string()),
            latitude: 45.0,
            longitude: -122.0,
            observed_on: Utc::now(),
            source: Source::GBIF,
            source_id: "test_1".to_string(),
            details: Some("Test sighting".to_string()),
        }
    }

    #[tokio::test]
    async fn test_broadcast_publish_subscribe() {
        let broadcast = Broadcast::new(32);
        let mut rx = broadcast.subscribe();

        let sighting = test_sighting();
        let event = StreamingEvent::SightingCreated(sighting.clone());
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
    async fn test_broadcast_multiple_subscribers() {
        let broadcast = Broadcast::new(32);
        let mut rx1 = broadcast.subscribe();
        let mut rx2 = broadcast.subscribe();

        let event = StreamingEvent::AnalysisComplete("done".to_string());
        broadcast.publish(event).unwrap();

        let received1 = rx1.recv().await.unwrap();
        let received2 = rx2.recv().await.unwrap();

        match (&received1, &received2) {
            (StreamingEvent::AnalysisComplete(m1), StreamingEvent::AnalysisComplete(m2)) => {
                assert_eq!(m1, "done");
                assert_eq!(m2, "done");
            }
            _ => panic!("Expected AnalysisComplete"),
        }
    }
}
