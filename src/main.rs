mod peaks;
mod tasks;

use std::ops::Deref;
use crate::tasks::{manage_tasks, Task};
use axum::extract::State;
use axum::{
    extract::DefaultBodyLimit,
    http::StatusCode,
    routing::{get, post},
    Router,
};
use axum_typed_multipart::{FieldData, TryFromMultipart, TypedMultipart};
use std::sync::{Arc};
use tokio::sync::Mutex;
use std::time::Duration;
use tempfile::NamedTempFile;

#[derive(TryFromMultipart)]
struct GenerateRequestDTO {
    #[form_data(limit = "50MB")]
    file: FieldData<NamedTempFile>,
    callback_api_url: String,
}

#[derive(Clone)]
struct AppState {
    tasks: Arc<Mutex<Vec<Task>>>,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_target(false)
        .compact()
        .init();

    let tasks_vec = Vec::<Task>::new();
    let tasks_mx = Mutex::new(tasks_vec);
    let mut tasks = Arc::new(tasks_mx);
    let state = AppState { tasks };

    let app = Router::new()
        .route("/", get(healthcheck))
        .route("/generate", post(generate))
        .layer(DefaultBodyLimit::max(1024 * 1024 * 64))
        .with_state(state.clone())
        .into_make_service();

    let listener = tokio::net::TcpListener::bind("0.0.0.0:4200").await.unwrap();

    tokio::spawn(async move {
        loop {
            println!("tasks len: {}", state.tasks.lock().await.len());
            manage_tasks(state.tasks.lock().await).await;

            tokio::time::sleep(Duration::from_secs(15)).await;
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
    let path = format!("tracks/{}.mp3", file_name); // UUID ?

    match file.contents.persist(&path) {
        Ok(_) => {}
        Err(_) => {
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    }

    let task: Task = Task::new(path.clone(), callback_api_url);
    state.tasks.lock().await.push(task);

    return Ok(StatusCode::OK);
}
