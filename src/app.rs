use crate::config::Config;
use crate::domain::{PipelineState, Project, ProjectId, StageId, StageState};
use crate::services::artifacts::{ArtifactStore, S3ArtifactStore};
use crate::services::orchestrator::DagScheduler;
use dashmap::DashMap;
use std::collections::HashMap;
use std::sync::Arc;

/// 애플리케이션 전역 상태
pub struct App {
    pub config: Config,
    pub projects: DashMap<uuid::Uuid, Project>,
    pub artifacts: Arc<dyn ArtifactStore>,
    pub cursor: Arc<crate::clients::cursor::CursorClient>,
    pub stitch: Arc<crate::clients::stitch::StitchClient>,
}

impl App {
    pub fn new(config: Config) -> crate::Result<Self> {
        let cursor = Arc::new(crate::clients::cursor::CursorClient::new(
            config.cursor_api_key.clone(),
        )?);
        let stitch = Arc::new(crate::clients::stitch::StitchClient::new(
            config.stitch_api_key.clone(),
        )?);
        let artifacts: Arc<dyn ArtifactStore> = Arc::new(S3ArtifactStore::new(
            &config.artifacts_endpoint,
            &config.artifacts_bucket,
        ));

        Ok(Self {
            config,
            projects: DashMap::new(),
            artifacts,
            cursor,
            stitch,
        })
    }

    pub fn create_project(&self, name: Option<String>, repo_url: Option<String>) -> Project {
        let id = ProjectId::new();
        let project = Project {
            id: id.clone(),
            name,
            repo_url,
            state: PipelineState::Pending,
            stages: StageId::all()
                .iter()
                .map(|&stage| (stage, StageState::Queued))
                .collect(),
            scheduler: DagScheduler::with_quality(id, self.config.max_debug_cycles),
            pdf_bytes: None,
            stage_outputs: HashMap::new(),
            accumulated_artifacts: Vec::new(),
        };
        self.projects.insert(project.id.0, project.clone());
        project
    }

    pub fn get_project(&self, id: uuid::Uuid) -> Option<Project> {
        self.projects.get(&id).map(|p| p.clone())
    }

    pub fn shared(self) -> Arc<Self> {
        Arc::new(self)
    }
}
