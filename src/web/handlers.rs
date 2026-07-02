use crate::app::App;
use crate::domain::{PipelineState, ProjectView, StageState};
use crate::error::{AutoForgeError, Result};
use crate::services::pipeline::{run_inline, start_project_mq};
use actix_multipart::Multipart;
use actix_web::http::header;
use actix_web::{web, HttpResponse};
use futures_util::{StreamExt, TryStreamExt};
use std::sync::Arc;
use uuid::Uuid;

pub async fn health(app: web::Data<Arc<App>>) -> HttpResponse {
    HttpResponse::Ok().json(serde_json::json!({
        "status": "ok",
        "service": "autoforge",
        "message_queue": app.queue.is_some(),
        "slack": app.slack.is_some(),
    }))
}

pub async fn index() -> HttpResponse {
    HttpResponse::Found()
        .append_header((header::LOCATION, "/static/index.html"))
        .finish()
}

pub async fn list_projects(app: web::Data<Arc<App>>) -> Result<HttpResponse> {
    let projects = app.store.list().await?;
    let views: Vec<ProjectView> = projects.iter().map(ProjectView::from).collect();
    Ok(HttpResponse::Ok().json(views))
}

pub async fn get_project(app: web::Data<Arc<App>>, path: web::Path<Uuid>) -> Result<HttpResponse> {
    let id = path.into_inner();
    let project = app
        .get_project(id)
        .await
        .ok_or_else(|| AutoForgeError::NotFound(format!("project {id}")))?;
    Ok(HttpResponse::Ok().json(ProjectView::from(&project)))
}

pub async fn create_project(
    app: web::Data<Arc<App>>,
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

    let pdf =
        pdf_bytes.ok_or_else(|| AutoForgeError::BadRequest("PDF file required (field: plan)".into()))?;

    if !pdf.starts_with(b"%PDF") {
        return Err(AutoForgeError::BadRequest("invalid PDF file".into()));
    }

    let mut project = app.create_project(name, repo_url).await;
    project.pdf_bytes = Some(pdf);
    project.state = PipelineState::Running;
    let project_id = project.id.0;
    app.store.save(&project).await?;

    if app.config.message_queue_enabled() && app.queue.is_some() {
        start_project_mq(app.get_ref().clone(), project_id).await?;
    } else {
        let app_clone = app.get_ref().clone();
        tokio::spawn(async move {
            if let Err(e) = run_inline(app_clone, project_id).await {
                tracing::error!(%project_id, error = %e, "pipeline failed");
            }
        });
    }

    let project = app
        .get_project(project_id)
        .await
        .ok_or_else(|| AutoForgeError::Internal("project lost after create".into()))?;

    Ok(HttpResponse::Accepted().json(serde_json::json!({
        "id": project_id,
        "state": project.state,
        "message": if app.queue.is_some() { "pipeline queued" } else { "pipeline started" },
        "mode": if app.queue.is_some() { "message_queue" } else { "inline" },
        "stream_url": format!("/v1/projects/{project_id}/stream"),
        "progress_percent": project.progress_percent(),
    })))
}

pub async fn stream_project(
    app: web::Data<Arc<App>>,
    path: web::Path<Uuid>,
) -> Result<HttpResponse> {
    let id = path.into_inner();
    let project = app
        .get_project(id)
        .await
        .ok_or_else(|| AutoForgeError::NotFound(format!("project {id}")))?;

    let stages: Vec<_> = crate::domain::StageId::all()
        .iter()
        .map(|stage| {
            let status = project
                .stages
                .get(stage)
                .copied()
                .unwrap_or(StageState::Queued);
            serde_json::json!({
                "stage": stage.as_str(),
                "status": format!("{status:?}").to_lowercase(),
            })
        })
        .collect();

    let body = format!(
        "event: status\ndata: {}\n\n",
        serde_json::json!({
            "project_id": id,
            "state": project.state,
            "progress_percent": project.progress_percent(),
            "stages": stages,
        })
    );

    Ok(HttpResponse::Ok()
        .content_type("text/event-stream")
        .insert_header((header::CACHE_CONTROL, "no-cache"))
        .insert_header((header::CONNECTION, "keep-alive"))
        .body(body))
}

pub async fn cancel_project(
    app: web::Data<Arc<App>>,
    path: web::Path<Uuid>,
) -> Result<HttpResponse> {
    let id = path.into_inner();
    let mut project = app
        .get_project(id)
        .await
        .ok_or_else(|| AutoForgeError::NotFound(format!("project {id}")))?;

    project.state = PipelineState::Cancelled;
    app.store.save(&project).await?;

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "id": id,
        "state": "cancelled",
    })))
}
