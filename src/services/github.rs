use crate::app::App;
use crate::clients::github::{slugify_repo_name, CreatedRepo};
use crate::domain::{Project, StageId};
use crate::error::Result;
use tracing::{info, warn};

/// 프로젝트용 GitHub 프라이빗 레포가 없으면 자동 생성
pub async fn ensure_project_repo(app: &App, project: &mut Project) -> Result<Option<CreatedRepo>> {
    let github = match &app.github {
        Some(g) if g.is_configured() => g,
        _ => return Ok(None),
    };

    if project.repo_url.is_some() {
        return Ok(None);
    }

    let display = project.display_name();
    let suffix = &project.id.0.to_string()[..8];
    let repo_name = slugify_repo_name(&display, suffix);

    let created = github
        .create_private_repo(&repo_name, Some(&format!("AutoForge project: {display}")))
        .await?;

    project.repo_url = Some(created.html_url.clone());
    project.stage_outputs.insert(
        StageId::Ingest,
        serde_json::json!({
            "github_repo": created.full_name,
            "github_html_url": created.html_url,
            "github_clone_url": created.clone_url,
            "auto_created": true,
        }),
    );

    info!(
        project_id = %project.id.0,
        repo = %created.full_name,
        "auto-created private github repository"
    );

    Ok(Some(created))
}

/// SecurityPatch 통과 후 PR 자동 머지
pub async fn try_auto_merge_pr(app: &App, project: &mut Project) -> Result<()> {
    let github = match &app.github {
        Some(g) if g.is_configured() && g.auto_merge_enabled() => g,
        _ => return Ok(()),
    };

    let security_passed = project
        .stage_outputs
        .get(&StageId::SecurityPatch)
        .and_then(|m| m.get("passed"))
        .and_then(|v| v.as_bool())
        .unwrap_or(false);

    if !security_passed {
        return Ok(());
    }

    let repo_url = match &project.repo_url {
        Some(url) => url.clone(),
        None => return Ok(()),
    };

    let pr_url = project
        .stage_outputs
        .get(&StageId::Implement)
        .and_then(|m| m.get("pr_url"))
        .and_then(|v| v.as_str())
        .map(String::from);

    match github.auto_merge_pr(&repo_url, pr_url.as_deref()).await {
        Ok(result) => {
            project.stage_outputs.insert(
                StageId::Deliver,
                serde_json::json!({
                    "merge_status": if result.merged { "merged" } else { "skipped" },
                    "merge_message": result.message,
                    "merge_sha": result.sha,
                    "pr_url": pr_url,
                }),
            );
            if result.merged {
                info!(project_id = %project.id.0, "PR auto-merged successfully");
            }
        }
        Err(e) => {
            warn!(project_id = %project.id.0, error = %e, "PR auto-merge failed");
            project.stage_outputs.insert(
                StageId::Deliver,
                serde_json::json!({
                    "merge_status": "failed",
                    "merge_message": e.to_string(),
                    "pr_url": pr_url,
                }),
            );
        }
    }

    Ok(())
}
