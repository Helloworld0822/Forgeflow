use super::ProjectStore;
use crate::domain::Project;
use crate::error::Result;
use async_trait::async_trait;
use dashmap::DashMap;
use uuid::Uuid;

pub struct MemoryStore {
    projects: DashMap<Uuid, Project>,
}

impl MemoryStore {
    pub fn new() -> Self {
        Self {
            projects: DashMap::new(),
        }
    }
}

impl Default for MemoryStore {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl ProjectStore for MemoryStore {
    async fn save(&self, project: &Project) -> Result<()> {
        self.projects.insert(project.id.0, project.clone());
        Ok(())
    }

    async fn get(&self, id: Uuid) -> Result<Option<Project>> {
        Ok(self.projects.get(&id).map(|p| p.clone()))
    }

    async fn list(&self) -> Result<Vec<Project>> {
        Ok(self.projects.iter().map(|e| e.value().clone()).collect())
    }
}
