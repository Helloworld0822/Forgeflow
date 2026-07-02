use crate::domain::{ProjectId, StageCommand, StageCompleted, StageId, StageState};
use crate::error::{AutoForgeError, Result};
use std::collections::{HashMap, HashSet};

/// DAG 스케줄러 — 순수 로직, I/O 없음
#[derive(Debug, Clone)]
pub struct DagScheduler {
    completed: HashSet<StageId>,
    running: HashSet<StageId>,
    failed: Option<StageId>,
    project_id: ProjectId,
}

impl DagScheduler {
    pub fn new() -> Self {
        Self {
            completed: HashSet::new(),
            running: HashSet::new(),
            failed: None,
            project_id: ProjectId::new(),
        }
    }

    pub fn with_project(project_id: ProjectId) -> Self {
        Self {
            project_id,
            ..Self::new()
        }
    }

    pub fn mark_running(&mut self, stage: StageId) {
        self.running.insert(stage);
    }

    pub fn mark_completed(&mut self, event: &StageCompleted) {
        self.running.remove(&event.stage);
        self.completed.insert(event.stage);
    }

    pub fn mark_failed(&mut self, stage: StageId) {
        self.running.remove(&stage);
        self.failed = Some(stage);
    }

    pub fn ready_stages(&self) -> Vec<StageCommand> {
        if self.failed.is_some() {
            return vec![];
        }

        let mut ready = Vec::new();

        if !self.completed.contains(&StageId::Ingest) && self.running.is_empty() {
            return vec![StageCommand {
                project_id: self.project_id.clone(),
                stage: StageId::Ingest,
                attempt: 0,
            }];
        }

        if self.completed.contains(&StageId::Ingest)
            && !self.completed.contains(&StageId::Summarize)
            && !self.running.contains(&StageId::Summarize)
        {
            ready.push(StageId::Summarize);
        }

        if self.completed.contains(&StageId::Summarize) {
            for stage in [StageId::Architect, StageId::Design] {
                if !self.completed.contains(&stage) && !self.running.contains(&stage) {
                    ready.push(stage);
                }
            }
        }

        let arch_done = self.completed.contains(&StageId::Architect);
        let design_done = self.completed.contains(&StageId::Design);
        if arch_done
            && design_done
            && !self.completed.contains(&StageId::Implement)
            && !self.running.contains(&StageId::Implement)
        {
            ready.push(StageId::Implement);
        }

        if self.completed.contains(&StageId::Implement)
            && !self.completed.contains(&StageId::Verify)
            && !self.running.contains(&StageId::Verify)
        {
            ready.push(StageId::Verify);
        }

        if self.completed.contains(&StageId::Verify)
            && !self.completed.contains(&StageId::Deliver)
            && !self.running.contains(&StageId::Deliver)
        {
            ready.push(StageId::Deliver);
        }

        ready
            .into_iter()
            .map(|stage| StageCommand {
                project_id: self.project_id.clone(),
                stage,
                attempt: 0,
            })
            .collect()
    }

    pub fn is_pipeline_complete(&self) -> bool {
        self.completed.contains(&StageId::Deliver)
    }
}

impl Default for DagScheduler {
    fn default() -> Self {
        Self::new()
    }
}

pub struct ProjectTracker {
    stages: HashMap<StageId, StageState>,
}

impl ProjectTracker {
    pub fn new() -> Self {
        Self {
            stages: HashMap::new(),
        }
    }

    pub fn transition(&mut self, stage: StageId, to: StageState) -> Result<()> {
        if let Some(current) = self.stages.get(&stage) {
            if !is_valid_transition(*current, to) {
                return Err(AutoForgeError::Orchestrator(format!(
                    "invalid transition for {stage:?}: {current:?} -> {to:?}"
                )));
            }
        }
        self.stages.insert(stage, to);
        Ok(())
    }
}

fn is_valid_transition(from: StageState, to: StageState) -> bool {
    matches!(
        (from, to),
        (StageState::Queued, StageState::Running)
            | (StageState::Running, StageState::Completed)
            | (StageState::Running, StageState::Failed)
            | (StageState::Failed, StageState::Queued)
            | (_, StageState::Skipped)
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parallel_architect_and_design_after_summarize() {
        let mut sched = DagScheduler::new();
        sched.mark_completed(&StageCompleted {
            project_id: ProjectId::new(),
            stage: StageId::Ingest,
            output_artifacts: vec![],
        });
        sched.mark_completed(&StageCompleted {
            project_id: ProjectId::new(),
            stage: StageId::Summarize,
            output_artifacts: vec![],
        });

        let ready: HashSet<_> = sched.ready_stages().into_iter().map(|c| c.stage).collect();
        assert!(ready.contains(&StageId::Architect));
        assert!(ready.contains(&StageId::Design));
    }

    #[test]
    fn implement_waits_for_both_architect_and_design() {
        let mut sched = DagScheduler::new();
        for stage in [StageId::Ingest, StageId::Summarize, StageId::Architect] {
            sched.mark_completed(&StageCompleted {
                project_id: ProjectId::new(),
                stage,
                output_artifacts: vec![],
            });
        }

        let ready: HashSet<_> = sched.ready_stages().into_iter().map(|c| c.stage).collect();
        assert!(!ready.contains(&StageId::Implement));
        assert!(ready.contains(&StageId::Design));
    }
}
