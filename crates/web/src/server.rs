use axum::{
    extract::{Path, Query, State},
    http::{StatusCode, header},
    response::{Json, Response, Html},
    routing::{get, Router},
    Request,
};
use neopilot_models_core::{Engine, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;
use tower::ServiceBuilder;
use tower_http::{
    cors::{Any, CorsLayer},
    services::ServeDir,
    compression::CompressionLayer,
};
use tracing::{info, error, warn};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[derive(Debug, Clone)]
pub struct AppState {
    pub engine: Arc<Engine>,
}

#[derive(Debug, Deserialize)]
pub struct SearchQuery {
    q: Option<String>,
    family: Option<String>,
    capability: Option<String>,
    modality: Option<String>,
    limit: Option<usize>,
    offset: Option<usize>,
}

#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub error: String,
    pub message: String,
}

#[derive(Debug, Serialize)]
pub struct StatsResponse {
    pub providers: usize,
    pub models: usize,
    pub families: HashMap<String, usize>,
    pub capabilities: HashMap<String, usize>,
    pub modalities: HashMap<String, usize>,
}

pub struct Server {
    app: Router,
}

impl Server {
    pub fn new(engine: Arc<Engine>, static_dir: Option<PathBuf>) -> Self {
        let state = AppState { engine };

        let mut app = Router::new()
            // API routes
            .route("/api.json", get(get_all_providers))
            .route("/api/providers", get(get_all_providers))
            .route("/api/providers/:id", get(get_provider))
            .route("/api/providers/:id/models/:model_id", get(get_model))
            .route("/api/models", get(search_models))
            .route("/api/models/schema", get(get_model_schema))
            .route("/api/models/ids", get(get_all_model_ids))
            .route("/api/stats", get(get_statistics))
            // Logo routes
            .route("/logos/:provider.svg", get(get_provider_logo))
            // Health check
            .route("/health", get(health_check))
            .with_state(state);

        // Add static file serving if directory is provided
        if let Some(dir) = static_dir {
            if dir.exists() {
                info!("Serving static files from: {:?}", dir);
                app = app.nest_service("/", ServeDir::new(dir).fallback(ServeDir::new(dir.join("index.html"))))
                    .layer(ServiceBuilder::new()
                        .layer(CompressionLayer::new())
                        .layer(CorsLayer::new()
                            .allow_origin(Any)
                            .allow_methods(Any)
                            .allow_headers(Any)));
            } else {
                warn!("Static directory {:?} does not exist", dir);
            }
        }

        Self { app }
    }

    pub async fn run(self, addr: SocketAddr) -> Result<()> {
        info!("Starting server on {}", addr);

        let listener = tokio::net::TcpListener::bind(addr)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to bind to {}: {}", addr, e))?;

        axum::serve(listener, self.app.into_make_service())
            .await
            .map_err(|e| anyhow::anyhow!("Server error: {}", e))?;

        Ok(())
    }
}

// API Handlers

async fn get_all_providers(
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.engine.get_providers_json() {
        Ok(data) => Ok(Json(data)),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

async fn get_provider(
    State(state): State<AppState>,
    Path(provider_id): Path<String>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorResponse>)> {
    match state.engine.get_provider_json(&provider_id) {
        Ok(data) => Ok(Json(data)),
        Err(e) => {
            let status = match e {
                neopilot_models_core::Error::ProviderNotFound(_) => StatusCode::NOT_FOUND,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            };
            let error_response = ErrorResponse {
                error: "provider_error".to_string(),
                message: e.to_string(),
            };
            Err((status, Json(error_response)))
        }
    }
}

async fn get_model(
    State(state): State<AppState>,
    Path((provider_id, model_id)): Path<(String, String)>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorResponse>)> {
    match state.engine.get_model_json(&provider_id, &model_id) {
        Ok(data) => Ok(Json(data)),
        Err(e) => {
            let status = match e {
                neopilot_models_core::Error::ModelNotFound(_) => StatusCode::NOT_FOUND,
                neopilot_models_core::Error::ProviderNotFound(_) => StatusCode::NOT_FOUND,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            };
            let error_response = ErrorResponse {
                error: "model_error".to_string(),
                message: e.to_string(),
            };
            Err((status, Json(error_response)))
        }
    }
}

async fn search_models(
    State(state): State<AppState>,
    Query(query): Query<SearchQuery>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let models = if let Some(search_query) = query.q {
        state.engine.search_models(&search_query)
    } else if let Some(family) = query.family {
        state.engine.get_models_by_family(&family)
    } else if let Some(capability) = query.capability {
        state.engine.get_models_by_capability(&capability)
    } else if let Some(modality) = query.modality {
        state.engine.get_models_by_modality(&modality)
    } else {
        // Return all models if no filter is specified
        state.engine.get_database().all_models().map(|(_, _, model)| model).collect()
    };

    let limit = query.limit.unwrap_or(100);
    let offset = query.offset.unwrap_or(0);

    let paginated_models: Vec<_> = models
        .into_iter()
        .skip(offset)
        .take(limit)
        .collect();

    let json_result = serde_json::to_value(&paginated_models)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(json_result))
}

async fn get_model_schema(
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.engine.get_model_schema() {
        Ok(schema) => {
            let json_schema = serde_json::to_value(schema)
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
            Ok(Json(json_schema))
        }
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

async fn get_all_model_ids(
    State(state): State<AppState>,
) -> Json<serde_json::Value> {
    let ids = state.engine.get_all_model_ids();
    Json(serde_json::to_value(ids).unwrap_or_default())
}

async fn get_statistics(
    State(state): State<AppState>,
) -> Json<serde_json::Value> {
    Json(state.engine.get_statistics())
}

async fn get_provider_logo(
    Path(provider_id): Path<String>,
) -> Result<Response, (StatusCode, Json<ErrorResponse>)> {
    let logo_path = PathBuf::from("providers")
        .join(&provider_id)
        .join("logo.svg");

    let default_logo_path = PathBuf::from("providers")
        .join("logo.svg");

    let final_path = if logo_path.exists() {
        logo_path
    } else if default_logo_path.exists() {
        default_logo_path
    } else {
        let error_response = ErrorResponse {
            error: "logo_not_found".to_string(),
            message: "Logo not found".to_string(),
        };
        return Err((StatusCode::NOT_FOUND, Json(error_response)));
    };

    match tokio::fs::read(&final_path).await {
        Ok(contents) => {
            let response = Response::builder()
                .status(StatusCode::OK)
                .header(header::CONTENT_TYPE, "image/svg+xml")
                .header(header::CACHE_CONTROL, "public, max-age=3600")
                .body(axum::body::Body::from(contents))
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
            Ok(response)
        }
        Err(_) => {
            let error_response = ErrorResponse {
                error: "logo_read_error".to_string(),
                message: "Failed to read logo file".to_string(),
            };
            Err((StatusCode::INTERNAL_SERVER_ERROR, Json(error_response)))
        }
    }
}

async fn health_check() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "status": "healthy",
        "timestamp": chrono::Utc::now().to_rfc3339()
    }))
}

// Error handling for the server
pub fn make_error_response(err: String) -> (StatusCode, Json<ErrorResponse>) {
    error!("Server error: {}", err);
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(ErrorResponse {
            error: "internal_error".to_string(),
            message: "An internal error occurred".to_string(),
        }),
    )
}
