use crate::app::App;
use crate::domain::{PipelineState, ProjectView, StageState};
use crate::error::{AutoForgeError, Result};
use crate::services::pipeline;
use actix_multipart::Multipart;
use actix_web::http::header;
use actix_web::{web, HttpResponse};
use futures_util::{StreamExt, TryStreamExt};
use uuid::Uuid;

pub async fn health() -> HttpResponse {
    HttpResponse::Ok().json(serde_json::json!({
        "status": "ok",
        "service": "autoforge",
    }))
}

pub async fn index() -> HttpResponse {
    HttpResponse::Found()
        .append_header((header::LOCATION, "/static/index.html"))
        .finish()
}

pub async fn list_projects(app: web::Data<App>) -> HttpResponse {
    let projects: Vec<ProjectView> = app
        .projects
        .iter()
        .map(|entry| ProjectView::from(entry.value()))
        .collect();
    HttpResponse::Ok().json(projects)
}

pub async fn get_project(app: web::Data<App>, path: web::Path<Uuid>) -> Result<HttpResponse> {
    let id = path.into_inner();
    let project = app
        .get_project(id)
        .ok_or_else(|| AutoForgeError::NotFound(format!("project {id}")))?;
    Ok(HttpResponse::Ok().json(ProjectView::from(&project)))
}

pub async fn create_project(
    app: web::Data<App>,
    mut payload: Multipart,
) -> Result<HttpResponse> {
    let mut name: Option<String> = None;
    let mut repo_url: Option<String> = None;
    let mut pdf_bytes: Option<Vec<u8>> = None;

    while let Some(field) = payload
        .try_next()
        .await
        .map_err(|e| AutoForgeError::BadRequest(e.to_string()))?
    {
        let field_name = field
            .content_disposition()
            .and_then(|d| d.get_name().map(String::from))
            .unwrap_or_default();

        let mut data = Vec::new();
        let mut field = field;
        while let Some(chunk) = field.next().await {
            let chunk = chunk.map_err(|e| AutoForgeError::BadRequest(e.to_string()))?;
            data.extend_from_slice(&chunk);
        }

        match field_name.as_str() {
            "name" => name = Some(String::from_utf8_lossy(&data).to_string()),
            "repo_url" => repo_url = Some(String::from_utf8_lossy(&data).to_string()),
            "plan" | "pdf" | "file" => pdf_bytes = Some(data),
            _ => {}
        }
    }

    let pdf = pdf_bytes.ok_or_else(|| AutoForgeError::BadRequest("PDF file required (field: plan)".into()))?;

    if !pdf.starts_with(b"%PDF") {
        return Err(AutoForgeError::BadRequest("invalid PDF file".into()));
    }

    let mut project = app.create_project(name, repo_url);
    project.pdf_bytes = Some(pdf);
    project.state = PipelineState::Running;
    let project_id = project.id.0;
    app.projects.insert(project_id, project);

    let app_clone = app.clone();
    tokio::spawn(async move {
        if let Err(e) = pipeline::run_pipeline(app_clone, project_id).await {
            tracing::error!(%project_id, error = %e, "pipeline failed");
        }
    });

    let project = app
        .get_project(project_id)
        .ok_or_else(|| AutoForgeError::Internal("project lost after create".into()))?;

    Ok(HttpResponse::Accepted().json(serde_json::json!({
        "id": project_id,
        "state": project.state,
        "message": "pipeline started",
        "stream_url": format!("/v1/projects/{project_id}/stream"),
    })))
}

pub async fn stream_project(
    app: web::Data<App>,
    path: web::Path<Uuid>,
) -> Result<HttpResponse> {
    let id = path.into_inner();
    let project = app
        .get_project(id)
        .ok_or_else(|| AutoForgeError::NotFound(format!("project {id}")))?;

    let mut events = Vec::new();
    for stage in crate::domain::StageId::all() {
        let status = project
            .stages
            .get(stage)
            .copied()
            .unwrap_or(StageState::Queued);
        events.push(serde_json::json!({
            "stage": stage.as_str(),
            "status": format!("{status:?}").to_lowercase(),
        }));
    }

    let body = format!(
        "event: status\ndata: {}\n\n",
        serde_json::json!({
            "project_id": id,
            "state": project.state,
            "stages": events,
        })
    );

    Ok(HttpResponse::Ok()
        .content_type("text/event-stream")
        .insert_header((header::CACHE_CONTROL, "no-cache"))
        .insert_header((header::CONNECTION, "keep-alive"))
        .body(body))
}

pub async fn cancel_project(
    app: web::Data<App>,
    path: web::Path<Uuid>,
) -> Result<HttpResponse> {
    let id = path.into_inner();
    let mut project = app
        .get_project(id)
        .ok_or_else(|| AutoForgeError::NotFound(format!("project {id}")))?;

    project.state = PipelineState::Cancelled;
    app.projects.insert(id, project);

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "id": id,
        "state": "cancelled",
    })))
}
