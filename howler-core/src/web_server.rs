use axum::{
    extract::{Path, State},
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::db::Database;
use crate::models::{Sighting, Source};
use crate::streaming::{Broadcast, StreamingEvent};
use crate::websocket::ws_handler;

#[derive(Clone)]
pub struct AppState {
    pub db: Arc<Database>,
    pub broadcast: Broadcast,
}

#[derive(Deserialize)]
pub struct CreateSighting {
    pub species: String,
    pub scientific_name: Option<String>,
    pub latitude: f64,
    pub longitude: f64,
    pub source: String,
    pub source_id: String,
    pub details: Option<String>,
}

#[derive(Serialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<String>,
}

pub fn build_router(state: AppState) -> Router {
    Router::new()
        .route("/api/sightings", get(get_sightings).post(create_sighting))
        .route(
            "/api/sightings/{id}",
            get(get_sighting_by_id).put(update_sighting),
        )
        .route("/api/analysis/{id}", post(run_analysis))
        .route("/api/export/{format}", get(export_sightings))
        .route("/ws/stream", get(ws_handler))
        .with_state(state)
}

pub async fn start_server(host: &str, port: u16, state: AppState) -> anyhow::Result<()> {
    let router = build_router(state);
    let addr = format!("{}:{}", host, port);
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, router).await?;
    Ok(())
}

async fn get_sightings(
    State(state): State<AppState>,
) -> Result<Json<ApiResponse<Vec<Sighting>>>, (StatusCode, Json<ApiResponse<()>>)> {
    match state.db.get_all_sightings() {
        Ok(sightings) => Ok(Json(ApiResponse {
            success: true,
            data: Some(sightings),
            error: None,
        })),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse {
                success: false,
                data: None,
                error: Some(e.to_string()),
            }),
        )),
    }
}

async fn get_sighting_by_id(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<Json<ApiResponse<Sighting>>, (StatusCode, Json<ApiResponse<()>>)> {
    let sightings = state.db.get_all_sightings().map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse {
                success: false,
                data: None,
                error: Some(e.to_string()),
            }),
        )
    })?;

    match sightings.into_iter().find(|s| s.id == Some(id)) {
        Some(sighting) => Ok(Json(ApiResponse {
            success: true,
            data: Some(sighting),
            error: None,
        })),
        None => Err((
            StatusCode::NOT_FOUND,
            Json(ApiResponse {
                success: false,
                data: None,
                error: Some("Sighting not found".to_string()),
            }),
        )),
    }
}

async fn create_sighting(
    State(state): State<AppState>,
    Json(input): Json<CreateSighting>,
) -> Result<(StatusCode, Json<ApiResponse<i64>>), (StatusCode, Json<ApiResponse<()>>)> {
    let now = chrono::Utc::now();
    let source = match input.source.as_str() {
        "GBIF" => Source::GBIF,
        "Movebank" => Source::Movebank,
        "iNaturalist" | "INaturalist" => Source::INaturalist,
        "IUCN" => Source::IUCN,
        _ => Source::GBIF,
    };

    let sighting = Sighting {
        id: None,
        species: input.species,
        scientific_name: input.scientific_name,
        latitude: input.latitude,
        longitude: input.longitude,
        observed_on: now,
        source,
        source_id: input.source_id,
        details: input.details,
    };

    match state.db.insert_sighting(&sighting) {
        Ok(id) => {
            let mut saved = sighting.clone();
            saved.id = Some(id);
            let _ = state.broadcast.publish(StreamingEvent::SightingCreated(saved));
            Ok((
                StatusCode::CREATED,
                Json(ApiResponse {
                    success: true,
                    data: Some(id),
                    error: None,
                }),
            ))
        }
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse {
                success: false,
                data: None,
                error: Some(e.to_string()),
            }),
        )),
    }
}

async fn update_sighting(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    Json(input): Json<CreateSighting>,
) -> Result<Json<ApiResponse<i64>>, (StatusCode, Json<ApiResponse<()>>)> {
    let now = chrono::Utc::now();
    let source = match input.source.as_str() {
        "GBIF" => Source::GBIF,
        "Movebank" => Source::Movebank,
        "iNaturalist" | "INaturalist" => Source::INaturalist,
        "IUCN" => Source::IUCN,
        _ => Source::GBIF,
    };

    let sighting = Sighting {
        id: Some(id),
        species: input.species,
        scientific_name: input.scientific_name,
        latitude: input.latitude,
        longitude: input.longitude,
        observed_on: now,
        source,
        source_id: input.source_id,
        details: input.details,
    };

    match state.db.insert_sighting(&sighting) {
        Ok(_inserted_id) => {
            let _ = state.broadcast.publish(StreamingEvent::SightingUpdated(sighting));
            Ok(Json(ApiResponse {
                success: true,
                data: Some(id),
                error: None,
            }))
        }
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse {
                success: false,
                data: None,
                error: Some(e.to_string()),
            }),
        )),
    }
}

async fn run_analysis(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<Json<ApiResponse<String>>, (StatusCode, Json<ApiResponse<()>>)> {
    let sightings = state.db.get_all_sightings().map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse {
                success: false,
                data: None,
                error: Some(e.to_string()),
            }),
        )
    })?;

    let analysis_id = format!("analysis_{}", id);
    let msg = format!(
        "Analysis complete for sighting {}: {} total sightings in dataset",
        id,
        sightings.len()
    );
    let _ = state
        .broadcast
        .publish(StreamingEvent::AnalysisComplete(msg.clone()));

    Ok(Json(ApiResponse {
        success: true,
        data: Some(analysis_id),
        error: None,
    }))
}

async fn export_sightings(
    State(state): State<AppState>,
    Path(format): Path<String>,
) -> Result<Json<ApiResponse<serde_json::Value>>, (StatusCode, Json<ApiResponse<()>>)> {
    let sightings = state.db.get_all_sightings().map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse {
                success: false,
                data: None,
                error: Some(e.to_string()),
            }),
        )
    })?;

    match format.as_str() {
        "json" => {
            let json_data = serde_json::to_value(&sightings).map_err(|e| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ApiResponse {
                        success: false,
                        data: None,
                        error: Some(e.to_string()),
                    }),
                )
            })?;
            Ok(Json(ApiResponse {
                success: true,
                data: Some(json_data),
                error: None,
            }))
        }
        _ => Err((
            StatusCode::BAD_REQUEST,
            Json(ApiResponse {
                success: false,
                data: None,
                error: Some(format!("Unsupported export format: {}", format)),
            }),
        )),
    }
}
