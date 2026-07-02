use crate::app::App;
use crate::domain::{ArtifactRef, PipelineState, StageCompleted, StageState};
use crate::error::{AutoForgeError, Result};
use crate::services::worker::{executors, StageContext};
use actix_web::web;
use bytes::Bytes;
use tracing::{info, warn};

/// 프로젝트 파이프라인을 백그라운드에서 실행
pub async fn run_pipeline(app: web::Data<App>, project_id: uuid::Uuid) -> Result<()> {
    let mut project = app
        .get_project(project_id)
        .ok_or_else(|| AutoForgeError::NotFound(format!("project {project_id}")))?;

    project.state = PipelineState::Running;
    app.projects.insert(project_id, project.clone());

    let pdf_key = format!("projects/{project_id}/plan.pdf");
    if let Some(pdf) = &project.pdf_bytes {
        app.artifacts
            .put(&pdf_key, Bytes::from(pdf.clone()), "application/pdf")
            .await?;
    }

    let input = vec![ArtifactRef {
        name: "plan.pdf".into(),
        uri: app.artifacts.uri_for(&pdf_key),
        content_type: "application/pdf".into(),
        sha256: None,
    }];

    let executor_map: std::collections::HashMap<_, _> = executors()
        .into_iter()
        .map(|e| (e.stage(), e))
        .collect();

    loop {
        let commands = project.scheduler.ready_stages();
        if commands.is_empty() {
            if project.scheduler.is_pipeline_complete() {
                project.state = PipelineState::Completed;
                app.projects.insert(project_id, project);
                info!(%project_id, "pipeline completed");
                return Ok(());
            }
            break;
        }

        for cmd in commands {
            let stage = cmd.stage;
            let executor = executor_map
                .get(&stage)
                .ok_or_else(|| AutoForgeError::Internal(format!("no executor for {stage:?}")))?;

            info!(%project_id, ?stage, "running stage");
            project.stages.insert(stage, StageState::Running);
            project.scheduler.mark_running(stage);
            app.projects.insert(project_id, project.clone());

            let ctx = StageContext {
                command: cmd.clone(),
                artifacts: app.artifacts.clone(),
                cursor: app.cursor.clone(),
                stitch: app.stitch.clone(),
                input: input.clone(),
                repo_url: project
                    .repo_url
                    .clone()
                    .or(app.config.default_repo_url.clone()),
            };

            match executor.execute(&ctx).await {
                Ok(output) => {
                    project.stages.insert(stage, StageState::Completed);
                    project.scheduler.mark_completed(&StageCompleted {
                        project_id: project.id.clone(),
                        stage,
                        output_artifacts: output.artifacts,
                    });
                }
                Err(e) => {
                    warn!(%project_id, ?stage, error = %e, "stage failed");
                    project.stages.insert(stage, StageState::Failed);
                    project.scheduler.mark_failed(stage);
                    project.state = PipelineState::Failed;
                    app.projects.insert(project_id, project);
                    return Err(e);
                }
            }

            app.projects.insert(project_id, project.clone());
        }
    }

    Ok(())
}
