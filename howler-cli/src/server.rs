use anyhow::Result;
use howler_core::{Broadcast, Database};
use std::sync::Arc;

pub async fn run_server(port: u16) -> Result<()> {
    let db = Database::new("howler.db")?;
    let broadcast = Broadcast::new(256);

    let state = howler_core::web_server::AppState {
        db: Arc::new(db),
        broadcast,
    };

    println!("Starting Howler server on 0.0.0.0:{}", port);
    howler_core::web_server::start_server("0.0.0.0", port, state).await
}
