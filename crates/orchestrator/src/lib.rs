use autoforge_shared::{
    AutoForgeError, ProjectId, Result, StageCommand, StageCompleted, StageId, StageState,
};
use std::collections::{HashMap, HashSet};

/// Pure DAG scheduler — no I/O, fully testable.
pub struct DagScheduler {
    completed: HashSet<StageId>,
    running: HashSet<StageId>,
    failed: Option<StageId>,
}

impl DagScheduler {
    pub fn new() -> Self {
        Self {
            completed: HashSet::new(),
            running: HashSet::new(),
            failed: None,
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

    /// Returns stages ready to enqueue based on DAG dependencies.
    pub fn ready_stages(&self) -> Vec<StageCommand> {
        if self.failed.is_some() {
            return vec![];
        }

        let mut ready = Vec::new();

        if !self.completed.contains(&StageId::Ingest) && self.running.is_empty() {
            return vec![StageCommand {
                project_id: ProjectId::new(),
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
                project_id: ProjectId::new(),
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

/// Tracks per-project stage states in memory.
/// Production: backed by PostgreSQL.
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

    pub fn state(&self, stage: StageId) -> Option<StageState> {
        self.stages.get(&stage).copied()
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
        for stage in [
            StageId::Ingest,
            StageId::Summarize,
            StageId::Architect,
        ] {
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
