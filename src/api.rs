use crate::db::AssetDb;
use axum::{extract::State, http::Method, routing::get, Json, Router};
use serde::Serialize;
use std::sync::Arc;
use tower_http::cors::{Any, CorsLayer};

#[derive(Clone)]
pub struct ApiState {
    pub db: Arc<AssetDb>,
}

#[derive(Serialize)]
pub struct StatusResponse {
    pub version: &'static str,
    pub name: &'static str,
    pub asset_counts: std::collections::HashMap<String, u32>,
    pub recent_runs: Vec<crate::db::PipelineRun>,
}

/// Start the embedded HTTP API server
pub async fn serve(db: AssetDb, port: u16) -> crate::Result<()> {
    let state = ApiState { db: Arc::new(db) };

    let cors = CorsLayer::new()
        .allow_methods([Method::GET, Method::POST])
        .allow_origin(Any)
        .allow_headers(Any);

    let app = Router::new()
        .route("/api/v1/status", get(handle_status))
        .route("/api/v1/assets", get(handle_assets))
        .route("/api/v1/pipeline-runs", get(handle_pipeline_runs))
        .route("/api/v1/errors", get(handle_errors))
        .layer(cors)
        .with_state(state);

    let addr = format!("127.0.0.1:{}", port);
    tracing::info!("📡 API server listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

async fn handle_status(State(state): State<ApiState>) -> Json<StatusResponse> {
    let asset_counts = state.db.get_asset_counts().unwrap_or_default();
    let recent_runs = state.db.get_recent_runs(10).unwrap_or_default();

    Json(StatusResponse {
        version: "0.1.0",
        name: "Rift Pipeline Engine",
        asset_counts,
        recent_runs,
    })
}

#[derive(Serialize)]
pub struct AssetListResponse {
    assets: Vec<crate::db::AssetRecord>,
    total: usize,
}

async fn handle_assets(State(state): State<ApiState>) -> Json<AssetListResponse> {
    let assets = state.db.get_assets(None, 100, 0).unwrap_or_default();
    let total = assets.len();
    Json(AssetListResponse { assets, total })
}

#[derive(Serialize)]
pub struct RunsResponse {
    runs: Vec<crate::db::PipelineRun>,
}

async fn handle_pipeline_runs(State(state): State<ApiState>) -> Json<RunsResponse> {
    let runs = state.db.get_recent_runs(20).unwrap_or_default();
    Json(RunsResponse { runs })
}

#[derive(Serialize)]
pub struct ErrorsResponse {
    errors: Vec<serde_json::Value>,
}

async fn handle_errors(State(state): State<ApiState>) -> Json<ErrorsResponse> {
    let errors = state.db.get_asset_errors(100).unwrap_or_default();
    Json(ErrorsResponse { errors })
}
