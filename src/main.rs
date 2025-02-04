mod peaks;
mod tasks;

use crate::tasks::{manage_tasks, remove_finished_tasks, Task};
use axum::extract::State;
use axum::response::IntoResponse;
use axum::{
    extract::DefaultBodyLimit,
    http::StatusCode,
    routing::{get, post},
    Router, ServiceExt,
};
use axum_typed_multipart::{FieldData, TryFromMultipart, TypedMultipart};
use serde::Serialize;
use sqlx::sqlite::SqlitePoolOptions;
use sqlx::{Executor, Pool, Sqlite};
use std::fs;
use std::sync::Arc;
use std::time::Duration;
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
    callback_api_url: String,
}

#[derive(Clone)]
struct AppState {
    pool: Arc<Pool<Sqlite>>,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_target(false)
        .compact()
        .init();

    let env = fs::read_to_string(".env").unwrap();
    let (_, database_url) = env.split_once('=').unwrap();

    let pool = match SqlitePoolOptions::new()
        .max_connections(50)
        .connect(&database_url)
        .await
    {
        Ok(pool) => pool,
        Err(e) => panic!("{}", e),
    };

    let arc = Arc::new(pool);
    let state = AppState { pool: arc.clone() };

    remove_finished_tasks(arc.clone()).await;

    let app = Router::new()
        .route("/", get(healthcheck))
        .route("/generate", post(generate))
        .layer(DefaultBodyLimit::max(1024 * 1024 * 64))
        .with_state(state)
        .into_make_service();

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8000").await.unwrap();

    tokio::spawn(async move {
        loop {
            let pool = arc.clone();
            manage_tasks(pool).await;

            tokio::time::sleep(Duration::from_secs(10)).await;
        }
    });

    axum::serve(listener, app).await.unwrap();
}

async fn healthcheck() -> &'static str {
    "ok"
}

#[axum::debug_handler]
async fn generate(
    State(state): State<AppState>,
    TypedMultipart(GenerateRequestDTO {
        file,
        callback_api_url,
    }): TypedMultipart<GenerateRequestDTO>,
) -> Result<StatusCode, StatusCode> {
    let file_name = file.metadata.file_name.unwrap_or("temp".to_string());
    let path = format!("tracks/{}", file_name);

    match file.contents.persist(&path) {
        Ok(_) => {}
        Err(_) => {
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    }

    let pool = state.pool;

    let mut task = Task::new(path.clone(), callback_api_url);
    match task.persist(pool).await {
        Ok(id) => {
            println!("Task persisted with id: {}", id);
        }
        Err(e) => {
            println!("Failed to persist task: {}", e);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    }

    return Ok(StatusCode::OK);
}
