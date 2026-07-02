use crate::clients::cursor::{CreateAgentOpts, CursorClient};
use crate::clients::stitch::StitchClient;
use crate::domain::{ArtifactRef, ModelProfile, StageCommand, StageId};
use crate::error::{AutoForgeError, Result};
use crate::services::artifacts::ArtifactStore;
use crate::services::ingest::ingest_pdf;
use async_trait::async_trait;
use bytes::Bytes;
use std::sync::Arc;

pub struct StageContext {
    pub command: StageCommand,
    pub artifacts: Arc<dyn ArtifactStore>,
    pub cursor: Arc<CursorClient>,
    pub stitch: Arc<StitchClient>,
    pub input: Vec<ArtifactRef>,
    pub repo_url: Option<String>,
}

#[derive(Debug)]
pub struct StageOutput {
    pub artifacts: Vec<ArtifactRef>,
    pub metadata: serde_json::Value,
}

#[async_trait]
pub trait StageExecutor: Send + Sync {
    fn stage(&self) -> StageId;
    async fn execute(&self, ctx: &StageContext) -> Result<StageOutput>;
}

pub struct IngestExecutor;

#[async_trait]
impl StageExecutor for IngestExecutor {
    fn stage(&self) -> StageId {
        StageId::Ingest
    }

    async fn execute(&self, ctx: &StageContext) -> Result<StageOutput> {
        let pdf_ref = ctx
            .input
            .iter()
            .find(|a| a.name.ends_with(".pdf"))
            .ok_or_else(|| AutoForgeError::Ingest("missing PDF input".into()))?;

        let bytes = ctx.artifacts.get(&pdf_ref.name).await?;
        let result = ingest_pdf(&bytes)?;
        let base = format!("projects/{}/ingest", ctx.command.project_id.0);

        let text_uri = ctx
            .artifacts
            .put(
                &format!("{base}/raw_text.md"),
                Bytes::from(result.raw_text.clone()),
                "text/markdown",
            )
            .await?;

        let meta = serde_json::json!({
            "page_count": result.page_count,
            "sha256": result.sha256,
        });
        let meta_uri = ctx
            .artifacts
            .put(
                &format!("{base}/ingest_meta.json"),
                Bytes::from(meta.to_string()),
                "application/json",
            )
            .await?;

        Ok(StageOutput {
            artifacts: vec![text_uri, meta_uri],
            metadata: meta,
        })
    }
}

pub struct SummarizeExecutor;

#[async_trait]
impl StageExecutor for SummarizeExecutor {
    fn stage(&self) -> StageId {
        StageId::Summarize
    }

    async fn execute(&self, ctx: &StageContext) -> Result<StageOutput> {
        let prompt = build_summarize_prompt(&ctx.input);
        let profile = ModelProfile::summarize();

        let resp = ctx
            .cursor
            .create_agent(&prompt, &profile, CreateAgentOpts::default())
            .await?;

        let run = ctx
            .cursor
            .wait_for_run(
                &resp.agent.id,
                &resp.run.id,
                std::time::Duration::from_secs(5),
            )
            .await?;

        let text = run
            .result
            .and_then(|r| r.text)
            .ok_or_else(|| AutoForgeError::StageFailed {
                stage: StageId::Summarize,
                message: "empty agent response".into(),
            })?;

        let base = format!("projects/{}/summarize", ctx.command.project_id.0);
        let artifact = ctx
            .artifacts
            .put(
                &format!("{base}/summary.json"),
                Bytes::from(text),
                "application/json",
            )
            .await?;

        Ok(StageOutput {
            artifacts: vec![artifact],
            metadata: serde_json::json!({ "cursor_agent_id": resp.agent.id }),
        })
    }
}

pub struct ArchitectExecutor;

#[async_trait]
impl StageExecutor for ArchitectExecutor {
    fn stage(&self) -> StageId {
        StageId::Architect
    }

    async fn execute(&self, ctx: &StageContext) -> Result<StageOutput> {
        let prompt = build_architect_prompt(&ctx.input);
        let profile = ModelProfile::architect();

        let resp = ctx
            .cursor
            .create_agent(&prompt, &profile, CreateAgentOpts::default())
            .await?;

        let run = ctx
            .cursor
            .wait_for_run(
                &resp.agent.id,
                &resp.run.id,
                std::time::Duration::from_secs(10),
            )
            .await?;

        let text = run.result.and_then(|r| r.text).unwrap_or_default();
        let base = format!("projects/{}/architect", ctx.command.project_id.0);
        let spec = ctx
            .artifacts
            .put(
                &format!("{base}/spec.md"),
                Bytes::from(text),
                "text/markdown",
            )
            .await?;

        Ok(StageOutput {
            artifacts: vec![spec],
            metadata: serde_json::json!({ "cursor_agent_id": resp.agent.id }),
        })
    }
}

pub struct DesignExecutor;

#[async_trait]
impl StageExecutor for DesignExecutor {
    fn stage(&self) -> StageId {
        StageId::Design
    }

    async fn execute(&self, ctx: &StageContext) -> Result<StageOutput> {
        let prompt = build_design_prompt(&ctx.input);
        let screen = ctx.stitch.generate_screen(&prompt, "DESKTOP").await?;
        let html = ctx.stitch.get_screen_html(&screen.id).await?;

        let artifact = ArtifactRef {
            name: format!("screens/{}.html", screen.id),
            uri: html.download_url,
            content_type: "text/html".into(),
            sha256: None,
        };

        Ok(StageOutput {
            artifacts: vec![artifact],
            metadata: serde_json::json!({ "screen_id": screen.id }),
        })
    }
}

pub struct ImplementExecutor;

#[async_trait]
impl StageExecutor for ImplementExecutor {
    fn stage(&self) -> StageId {
        StageId::Implement
    }

    async fn execute(&self, ctx: &StageContext) -> Result<StageOutput> {
        let repo_url = ctx
            .repo_url
            .as_deref()
            .ok_or_else(|| AutoForgeError::BadRequest("repo_url required".into()))?;

        let prompt = build_implement_prompt(&ctx.input);
        let profile = ModelProfile::implement();

        let opts = CreateAgentOpts {
            repo_url: Some(repo_url),
            starting_ref: Some("main"),
            auto_create_pr: Some(true),
            agent_id: None,
        };

        let resp = ctx.cursor.create_agent(&prompt, &profile, opts).await?;
        let run = ctx
            .cursor
            .wait_for_run(
                &resp.agent.id,
                &resp.run.id,
                std::time::Duration::from_secs(15),
            )
            .await?;

        let pr_url = run
            .result
            .and_then(|r| r.git)
            .and_then(|g| g.branches)
            .and_then(|b| b.into_iter().next())
            .and_then(|br| br.pr_url);

        Ok(StageOutput {
            artifacts: vec![],
            metadata: serde_json::json!({
                "cursor_agent_id": resp.agent.id,
                "pr_url": pr_url,
            }),
        })
    }
}

pub fn executors() -> Vec<Arc<dyn StageExecutor>> {
    vec![
        Arc::new(IngestExecutor),
        Arc::new(SummarizeExecutor),
        Arc::new(ArchitectExecutor),
        Arc::new(DesignExecutor),
        Arc::new(ImplementExecutor),
    ]
}

fn build_summarize_prompt(inputs: &[ArtifactRef]) -> String {
    format!(
        "다음 외주 계획서를 분석하여 strict JSON으로 요약하세요.\n\
         필드: title, goals[], scope, constraints[], tech_hints[], ui_requirements[], timeline, budget_hint\n\
         입력 아티팩트: {:?}",
        inputs.iter().map(|a| &a.uri).collect::<Vec<_>>()
    )
}

fn build_architect_prompt(inputs: &[ArtifactRef]) -> String {
    format!(
        "summary.json을 기반으로 시스템 아키텍처(architecture.md)와 상세 기획(spec.md), \
         구현 태스크 DAG(tasks.json)를 작성하세요.\n\
         입력: {:?}",
        inputs.iter().map(|a| &a.uri).collect::<Vec<_>>()
    )
}

fn build_design_prompt(inputs: &[ArtifactRef]) -> String {
    format!(
        "ui_requirements를 반영한 모던 UI 대시보드를 디자인하세요.\n\
         입력: {:?}",
        inputs.iter().map(|a| &a.uri).collect::<Vec<_>>()
    )
}

fn build_implement_prompt(inputs: &[ArtifactRef]) -> String {
    format!(
        "tasks.json 순서대로 구현하세요. design/screens/ 의 Stitch HTML을 UI 참고로 사용하세요.\n\
         입력: {:?}",
        inputs.iter().map(|a| &a.uri).collect::<Vec<_>>()
    )
}
