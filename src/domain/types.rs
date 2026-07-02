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
    Debug,
    SecurityPatch,
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
            StageId::Debug,
            StageId::SecurityPatch,
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
            StageId::Debug => "debug",
            StageId::SecurityPatch => "security_patch",
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

    pub fn verify() -> Self {
        Self {
            model_id: "gpt-5.3-codex-high".into(),
            mode: AgentMode::Agent,
            params: vec![],
        }
    }

    pub fn debug() -> Self {
        Self {
            model_id: "gpt-5.3-codex-high".into(),
            mode: AgentMode::Agent,
            params: vec![],
        }
    }

    pub fn security_patch() -> Self {
        Self {
            model_id: "claude-fable-5-thinking-high".into(),
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

/// 런타임 프로젝트 엔티티
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    pub id: ProjectId,
    pub name: Option<String>,
    pub repo_url: Option<String>,
    pub state: PipelineState,
    pub stages: HashMap<StageId, StageState>,
    pub scheduler: DagScheduler,
    pub pdf_bytes: Option<Vec<u8>>,
    /// 스테이지별 메타데이터 (pr_url, verify_report 등)
    pub stage_outputs: HashMap<StageId, serde_json::Value>,
    /// 누적 산출물 참조
    pub accumulated_artifacts: Vec<ArtifactRef>,
    /// Slack 진행 메시지 ts (업데이트용)
    #[serde(default)]
    pub slack_message_ts: Option<String>,
}

impl Project {
    /// 전체 파이프라인 진행률 (0–100)
    pub fn progress_percent(&self) -> u8 {
        let total = StageId::all().len() as u32;
        let done = self
            .stages
            .values()
            .filter(|s| matches!(s, StageState::Completed | StageState::Skipped))
            .count() as u32;
        ((done * 100) / total.max(1)) as u8
    }

    pub fn display_name(&self) -> String {
        self.name
            .clone()
            .unwrap_or_else(|| format!("Project {}", &self.id.0.to_string()[..8]))
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct ProjectView {
    pub id: Uuid,
    pub name: Option<String>,
    pub repo_url: Option<String>,
    pub state: PipelineState,
    pub stages: Vec<StageStatusView>,
    pub progress_percent: u8,
    pub pr_url: Option<String>,
    pub merge_status: Option<String>,
    pub github_repo: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize)]
pub struct StageStatusView {
    pub stage: StageId,
    pub status: StageState,
}

impl From<&Project> for ProjectView {
    fn from(p: &Project) -> Self {
        let pr_url = p
            .stage_outputs
            .get(&StageId::Implement)
            .and_then(|m| m.get("pr_url"))
            .and_then(|v| v.as_str())
            .map(String::from);

        let merge_status = p
            .stage_outputs
            .get(&StageId::Deliver)
            .and_then(|m| m.get("merge_status"))
            .and_then(|v| v.as_str())
            .map(String::from);

        let github_repo = p
            .stage_outputs
            .get(&StageId::Ingest)
            .and_then(|m| m.get("github_repo"))
            .and_then(|v| v.as_str())
            .map(String::from);

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
            progress_percent: p.progress_percent(),
            pr_url,
            merge_status,
            github_repo,
            created_at: Utc::now(),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct ProjectDetailView {
    #[serde(flatten)]
    pub view: ProjectView,
    pub stage_outputs: HashMap<String, serde_json::Value>,
}

impl From<&Project> for ProjectDetailView {
    fn from(p: &Project) -> Self {
        let stage_outputs = p
            .stage_outputs
            .iter()
            .map(|(k, v)| (k.as_str().to_string(), v.clone()))
            .collect();
        Self {
            view: ProjectView::from(p),
            stage_outputs,
        }
    }
}
