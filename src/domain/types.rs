use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

use crate::services::orchestrator::DagScheduler;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StageId {
    Ingest,
    Summarize,
    Architect,
    Design,
    Implement,
    Verify,
    Deliver,
}

impl StageId {
    pub fn all() -> &'static [StageId] {
        &[
            StageId::Ingest,
            StageId::Summarize,
            StageId::Architect,
            StageId::Design,
            StageId::Implement,
            StageId::Verify,
            StageId::Deliver,
        ]
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            StageId::Ingest => "ingest",
            StageId::Summarize => "summarize",
            StageId::Architect => "architect",
            StageId::Design => "design",
            StageId::Implement => "implement",
            StageId::Verify => "verify",
            StageId::Deliver => "deliver",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AgentMode {
    Agent,
    Plan,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelProfile {
    pub model_id: String,
    pub mode: AgentMode,
    #[serde(default)]
    pub params: Vec<ModelParam>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelParam {
    pub id: String,
    pub value: String,
}

impl ModelProfile {
    pub fn summarize() -> Self {
        Self {
            model_id: "claude-4.6-sonnet-high-thinking".into(),
            mode: AgentMode::Agent,
            params: vec![],
        }
    }

    pub fn architect() -> Self {
        Self {
            model_id: "claude-fable-5-thinking-high".into(),
            mode: AgentMode::Plan,
            params: vec![],
        }
    }

    pub fn implement() -> Self {
        Self {
            model_id: "gpt-5.3-codex-high".into(),
            mode: AgentMode::Agent,
            params: vec![],
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PipelineState {
    Pending,
    Running,
    Completed,
    Failed,
    Cancelled,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectId(pub Uuid);

impl ProjectId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunId(pub Uuid);

impl RunId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StageState {
    Queued,
    Running,
    Completed,
    Failed,
    Skipped,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArtifactRef {
    pub name: String,
    pub uri: String,
    pub content_type: String,
    pub sha256: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StageCommand {
    pub project_id: ProjectId,
    pub stage: StageId,
    pub attempt: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StageCompleted {
    pub project_id: ProjectId,
    pub stage: StageId,
    pub output_artifacts: Vec<ArtifactRef>,
}

/// 런타임 프로젝트 엔티티 (인메모리 저장)
#[derive(Debug, Clone)]
pub struct Project {
    pub id: ProjectId,
    pub name: Option<String>,
    pub repo_url: Option<String>,
    pub state: PipelineState,
    pub stages: HashMap<StageId, StageState>,
    pub scheduler: DagScheduler,
    pub pdf_bytes: Option<Vec<u8>>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ProjectView {
    pub id: Uuid,
    pub name: Option<String>,
    pub repo_url: Option<String>,
    pub state: PipelineState,
    pub stages: Vec<StageStatusView>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize)]
pub struct StageStatusView {
    pub stage: StageId,
    pub status: StageState,
}

impl From<&Project> for ProjectView {
    fn from(p: &Project) -> Self {
        Self {
            id: p.id.0,
            name: p.name.clone(),
            repo_url: p.repo_url.clone(),
            state: p.state,
            stages: StageId::all()
                .iter()
                .map(|&stage| StageStatusView {
                    stage,
                    status: p.stages.get(&stage).copied().unwrap_or(StageState::Queued),
                })
                .collect(),
            created_at: Utc::now(),
        }
    }
}
