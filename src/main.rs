mod peaks;

use crate::peaks::generate_peaks;
use axum::{
    extract::DefaultBodyLimit,
    http::StatusCode,
    routing::{get, post},
    Json, Router, ServiceExt,
};
use axum_typed_multipart::{FieldData, TryFromMultipart, TypedMultipart};
use serde::Serialize;
use std::fs;
use tempfile::NamedTempFile;

#[derive(Serialize)]
struct PeaksResponse {
    peaks: Vec<f32>,
}

#[derive(Serialize)]
struct FailedResponse {
    message: String,
}

#[derive(TryFromMultipart)]
struct GenerateRequestDTO {
    #[form_data(limit = "50MB")]
    file: FieldData<NamedTempFile>,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_target(false)
        .compact()
        .init();
    let app = Router::new()
        .route("/", get(healthcheck))
        .route("/generate", post(generate))
        .layer(DefaultBodyLimit::max(1024 * 1024 * 64))
        .into_make_service();

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

// basic handler that responds with a static string
async fn healthcheck() -> &'static str {
    "ok"
}

#[axum::debug_handler]
async fn generate(
    TypedMultipart(GenerateRequestDTO { file }): TypedMultipart<GenerateRequestDTO>,
) -> Result<Json<PeaksResponse>, StatusCode> {
    let file_name = file.metadata.file_name.unwrap_or("temp".to_string());
    let path = format!("tracks/{}", file_name);

    match file.contents.persist(&path) {
        Ok(_) => {}
        Err(_) => {
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    }

    let peaks = generate_peaks(path.clone());

    match fs::remove_file(path) {
        Ok(_) => {}
        Err(e) => {
            println!("Failed to remove a file: {e}");
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    return Ok(Json(PeaksResponse { peaks }));
}
