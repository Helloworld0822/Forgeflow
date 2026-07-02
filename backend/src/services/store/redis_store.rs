use super::ProjectStore;
use crate::domain::Project;
use crate::error::{AutoForgeError, Result};
use async_trait::async_trait;
use redis::aio::ConnectionManager;
use redis::AsyncCommands;
use uuid::Uuid;

const KEY_PREFIX: &str = "autoforge:project:";

pub struct RedisProjectStore {
    conn: ConnectionManager,
}

impl RedisProjectStore {
    pub async fn connect(redis_url: &str) -> Result<Self> {
        let client = redis::Client::open(redis_url)
            .map_err(|e| AutoForgeError::Store(e.to_string()))?;
        let conn = client
            .get_connection_manager()
            .await
            .map_err(|e| AutoForgeError::Store(e.to_string()))?;
        Ok(Self { conn })
    }

    fn key(id: Uuid) -> String {
        format!("{KEY_PREFIX}{id}")
    }
}

#[async_trait]
impl ProjectStore for RedisProjectStore {
    async fn save(&self, project: &Project) -> Result<()> {
        let mut conn = self.conn.clone();
        let json = serde_json::to_string(project)
            .map_err(|e| AutoForgeError::Store(e.to_string()))?;
        conn.set::<_, _, ()>(Self::key(project.id.0), json)
            .await
            .map_err(|e| AutoForgeError::Store(e.to_string()))?;
        Ok(())
    }

    async fn get(&self, id: Uuid) -> Result<Option<Project>> {
        let mut conn = self.conn.clone();
        let json: Option<String> = conn
            .get(Self::key(id))
            .await
            .map_err(|e| AutoForgeError::Store(e.to_string()))?;
        match json {
            Some(s) => {
                let p = serde_json::from_str(&s)
                    .map_err(|e| AutoForgeError::Store(e.to_string()))?;
                Ok(Some(p))
            }
            None => Ok(None),
        }
    }

    async fn list(&self) -> Result<Vec<Project>> {
        let mut conn = self.conn.clone();
        let keys: Vec<String> = conn
            .keys(format!("{KEY_PREFIX}*"))
            .await
            .map_err(|e| AutoForgeError::Store(e.to_string()))?;
        let mut projects = Vec::with_capacity(keys.len());
        for key in keys {
            let json: Option<String> = conn
                .get(&key)
                .await
                .map_err(|e| AutoForgeError::Store(e.to_string()))?;
            if let Some(s) = json {
                if let Ok(p) = serde_json::from_str(&s) {
                    projects.push(p);
                }
            }
        }
        Ok(projects)
    }
}
