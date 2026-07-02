use crate::domain::ArtifactRef;
use crate::error::{AutoForgeError, Result};
use async_trait::async_trait;
use bytes::Bytes;

#[async_trait]
pub trait ArtifactStore: Send + Sync {
    async fn put(&self, key: &str, data: Bytes, content_type: &str) -> Result<ArtifactRef>;
    async fn get(&self, key: &str) -> Result<Bytes>;
    fn uri_for(&self, key: &str) -> String;
}

pub struct S3ArtifactStore {
    endpoint: String,
    bucket: String,
    /// 인메모리 캐시 (개발용)
    cache: dashmap::DashMap<String, Bytes>,
}

impl S3ArtifactStore {
    pub fn new(endpoint: impl Into<String>, bucket: impl Into<String>) -> Self {
        Self {
            endpoint: endpoint.into(),
            bucket: bucket.into(),
            cache: dashmap::DashMap::new(),
        }
    }
}

#[async_trait]
impl ArtifactStore for S3ArtifactStore {
    async fn put(&self, key: &str, data: Bytes, content_type: &str) -> Result<ArtifactRef> {
        self.cache.insert(key.to_string(), data);
        Ok(ArtifactRef {
            name: key.to_string(),
            uri: self.uri_for(key),
            content_type: content_type.to_string(),
            sha256: None,
        })
    }

    async fn get(&self, key: &str) -> Result<Bytes> {
        self.cache
            .get(key)
            .map(|v| v.clone())
            .ok_or_else(|| AutoForgeError::Artifacts(format!("not found: {key}")))
    }

    fn uri_for(&self, key: &str) -> String {
        format!("{}/{}/{}", self.endpoint, self.bucket, key)
    }
}
