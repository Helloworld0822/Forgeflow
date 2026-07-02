use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

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

    /// Returns stages that can run in parallel after the given stage completes.
    pub fn next_parallel(&self) -> Vec<StageId> {
        match self {
            StageId::Ingest => vec![StageId::Summarize],
            StageId::Summarize => vec![StageId::Architect, StageId::Design],
            StageId::Architect | StageId::Design => vec![],
            StageId::Implement => vec![StageId::Verify],
            StageId::Verify => vec![StageId::Deliver],
            StageId::Deliver => vec![],
        }
    }

    /// Stages that must complete before `implement` can start.
    pub fn implement_prerequisites() -> &'static [StageId] {
        &[StageId::Architect, StageId::Design]
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

impl Default for ProjectId {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunId(pub Uuid);

impl RunId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StageRun {
    pub id: RunId,
    pub project_id: ProjectId,
    pub stage: StageId,
    pub state: StageState,
    pub cursor_agent_id: Option<String>,
    pub cursor_run_id: Option<String>,
    pub input_artifacts: Vec<ArtifactRef>,
    pub output_artifacts: Vec<ArtifactRef>,
    pub error: Option<String>,
    pub retry_count: u8,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
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
pub struct ProjectSummary {
    pub title: String,
    pub goals: Vec<String>,
    pub scope: String,
    pub constraints: Vec<String>,
    pub tech_hints: Vec<String>,
    pub ui_requirements: Vec<String>,
    pub timeline: Option<String>,
    pub budget_hint: Option<String>,
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
