use async_trait::async_trait;
use autoforge_shared::{ArtifactRef, AutoForgeError, Result};
use bytes::Bytes;

#[async_trait]
pub trait ArtifactStore: Send + Sync {
    async fn put(&self, key: &str, data: Bytes, content_type: &str) -> Result<ArtifactRef>;
    async fn get(&self, key: &str) -> Result<Bytes>;
    fn uri_for(&self, key: &str) -> String;
}

/// S3-compatible store using presigned-style URIs.
/// Production: swap for aws-sdk-s3 or MinIO client.
pub struct S3ArtifactStore {
    endpoint: String,
    bucket: String,
}

impl S3ArtifactStore {
    pub fn new(endpoint: impl Into<String>, bucket: impl Into<String>) -> Self {
        Self {
            endpoint: endpoint.into(),
            bucket: bucket.into(),
        }
    }
}

#[async_trait]
impl ArtifactStore for S3ArtifactStore {
    async fn put(&self, key: &str, _data: Bytes, content_type: &str) -> Result<ArtifactRef> {
        // TODO: implement actual S3 PUT via aws-sdk-s3
        Ok(ArtifactRef {
            name: key.to_string(),
            uri: self.uri_for(key),
            content_type: content_type.to_string(),
            sha256: None,
        })
    }

    async fn get(&self, _key: &str) -> Result<Bytes> {
        Err(AutoForgeError::Artifacts("not implemented".into()))
    }

    fn uri_for(&self, key: &str) -> String {
        format!("{}/{}/{}", self.endpoint, self.bucket, key)
    }
}
