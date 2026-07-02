use autoforge_shared::{PipelineState, ProjectId, StageId};
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::sse::{Event, Sse},
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::convert::Infallible;
use std::sync::Arc;
use tokio_stream::wrappers::ReceiverStream;
use tower_http::trace::TraceLayer;

#[derive(Clone)]
pub struct ApiState {
    // Production: inject orchestrator + artifact store
}

#[derive(Debug, Serialize)]
pub struct ProjectResponse {
    pub id: ProjectId,
    pub state: PipelineState,
    pub stages: Vec<StageStatus>,
}

#[derive(Debug, Serialize)]
pub struct StageStatus {
    pub stage: StageId,
    pub status: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateProjectRequest {
    pub name: Option<String>,
    pub repo_url: Option<String>,
}

pub fn router(state: ApiState) -> Router {
    Router::new()
        .route("/v1/projects", post(create_project))
        .route("/v1/projects/{id}", get(get_project))
        .route("/v1/projects/{id}/stream", get(stream_project))
        .route("/v1/projects/{id}/cancel", post(cancel_project))
        .layer(TraceLayer::new_for_http())
        .with_state(Arc::new(state))
}

async fn create_project(
    State(_state): State<Arc<ApiState>>,
    Json(_req): Json<CreateProjectRequest>,
) -> Result<Json<ProjectResponse>, StatusCode> {
    // TODO: multipart PDF upload + orchestrator enqueue
    let id = ProjectId::new();
    Ok(Json(ProjectResponse {
        id,
        state: PipelineState::Pending,
        stages: vec![],
    }))
}

async fn get_project(
    State(_state): State<Arc<ApiState>>,
    Path(_id): Path<uuid::Uuid>,
) -> Result<Json<ProjectResponse>, StatusCode> {
    Err(StatusCode::NOT_IMPLEMENTED)
}

async fn stream_project(
    State(_state): State<Arc<ApiState>>,
    Path(_id): Path<uuid::Uuid>,
) -> Sse<ReceiverStream<Result<Event, Infallible>>> {
    let (tx, rx) = tokio::sync::mpsc::channel(16);

    tokio::spawn(async move {
        let _ = tx
            .send(Ok(Event::default().data(r#"{"stage":"ingest","progress":0}"#)))
            .await;
    });

    Sse::new(ReceiverStream::new(rx))
}

async fn cancel_project(
    State(_state): State<Arc<ApiState>>,
    Path(_id): Path<uuid::Uuid>,
) -> StatusCode {
    StatusCode::NOT_IMPLEMENTED
}
