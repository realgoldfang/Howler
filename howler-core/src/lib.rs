pub mod annotations;
pub mod auth;
pub mod clustering;
pub mod config;
pub mod crypto;
pub mod db;
pub mod export;
pub mod filter;
pub mod import;
pub mod migrations;
pub mod models;
pub mod movement;
pub mod temporal;

#[cfg(feature = "server")]
pub mod streaming;
#[cfg(feature = "server")]
pub mod web_server;
#[cfg(feature = "server")]
pub mod websocket;

#[cfg(feature = "ml")]
pub mod ml;

#[cfg(feature = "api-fetch")]
pub mod gbif;
#[cfg(feature = "api-fetch")]
pub mod inaturalist;
#[cfg(feature = "api-fetch")]
pub mod iucn;
#[cfg(feature = "api-fetch")]
pub mod movebank;

#[cfg(test)]
mod tests;

uniffi::setup_scaffolding!();

pub use clustering::{ClusteringResult, DbscanParams, PackTerritory};
pub use config::Config;
pub use db::Database;
pub use export::ExportFormat;
pub use filter::SightingFilter;
pub use models::{MobileSighting, Sighting, Source};
pub use movement::{Movement, MovementAnalysis, MovementPattern};
pub use temporal::{ActivityStats, Season, TemporalAnalysis, TimePeriod};

#[cfg(feature = "ml")]
pub use ml::{
    generate_synthetic_training_data, predict_activity_pattern, ActivityPrediction,
    BehaviorFeatures, BehaviorModel, BehaviorPrediction, BehaviorType, LocationPrediction,
};

#[cfg(feature = "server")]
pub use streaming::{Broadcast, StreamingEvent};
#[cfg(feature = "server")]
pub use web_server::AppState;
