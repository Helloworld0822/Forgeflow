use crate::domain::ArtifactRef;
use crate::error::{AutoForgeError, Result};
use async_trait::async_trait;
use bytes::Bytes;
use s3::bucket::Bucket;
use s3::creds::Credentials;
use s3::region::Region;
use s3::BucketConfiguration;

#[async_trait]
pub trait ArtifactStore: Send + Sync {
    async fn put(&self, key: &str, data: Bytes, content_type: &str) -> Result<ArtifactRef>;
    async fn get(&self, key: &str) -> Result<Bytes>;
    fn uri_for(&self, key: &str) -> String;
    /// 실제 원격 스토리지(S3/MinIO)에 연결되어 있는지 여부.
    /// false면 프로세스 재시작/다중 인스턴스 환경에서 데이터가 유실될 수 있다.
    fn is_durable(&self) -> bool {
        true
    }
}

enum Backend {
    S3(Box<Bucket>),
    /// 개발용 인메모리 폴백. MinIO/S3에 연결할 수 없을 때만 사용되며
    /// 프로세스 재시작 시 및 다중 인스턴스(worker/orchestrator 분리) 환경에서
    /// 데이터가 공유/보존되지 않는다.
    Memory(dashmap::DashMap<String, Bytes>),
}

pub struct S3ArtifactStore {
    endpoint: String,
    bucket_name: String,
    backend: Backend,
}

impl S3ArtifactStore {
    /// 개발/테스트 전용 — 항상 인메모리로 동작한다.
    pub fn new_memory(endpoint: impl Into<String>, bucket: impl Into<String>) -> Self {
        Self {
            endpoint: endpoint.into(),
            bucket_name: bucket.into(),
            backend: Backend::Memory(dashmap::DashMap::new()),
        }
    }

    /// MinIO/S3 연결 시도 최대 대기 시간. 이 시간을 넘기면 인메모리 폴백으로 전환하여
    /// 스토리지 장애가 서버 기동 자체를 무기한 블록하지 않도록 한다.
    const CONNECT_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(8);

    /// MinIO/S3에 연결을 시도한다. 실패하거나 타임아웃되면 경고 로그를 남기고
    /// 인메모리 폴백으로 동작한다.
    pub async fn connect(
        endpoint: &str,
        bucket: &str,
        access_key: Option<&str>,
        secret_key: Option<&str>,
        region: &str,
    ) -> Self {
        let attempt = tokio::time::timeout(
            Self::CONNECT_TIMEOUT,
            Self::try_connect(endpoint, bucket, access_key, secret_key, region),
        )
        .await
        .unwrap_or_else(|_| {
            Err(AutoForgeError::Artifacts(format!(
                "connection attempt timed out after {:?}",
                Self::CONNECT_TIMEOUT
            )))
        });

        match attempt {
            Ok(b) => {
                tracing::info!(
                    endpoint,
                    bucket,
                    "artifact store: connected to S3-compatible backend"
                );
                Self {
                    endpoint: endpoint.into(),
                    bucket_name: bucket.into(),
                    backend: Backend::S3(b),
                }
            }
            Err(e) => {
                tracing::warn!(
                    endpoint,
                    bucket,
                    error = %e,
                    "artifact store: could not connect to S3/MinIO — falling back to in-memory \
                     (NOT durable, NOT shared across processes — do not use in distributed/production mode)"
                );
                Self {
                    endpoint: endpoint.into(),
                    bucket_name: bucket.into(),
                    backend: Backend::Memory(dashmap::DashMap::new()),
                }
            }
        }
    }

    async fn try_connect(
        endpoint: &str,
        bucket_name: &str,
        access_key: Option<&str>,
        secret_key: Option<&str>,
        region: &str,
    ) -> Result<Box<Bucket>> {
        let region = Region::Custom {
            region: region.to_string(),
            endpoint: endpoint.to_string(),
        };
        let credentials = Credentials::new(access_key, secret_key, None, None, None)
            .map_err(|e| AutoForgeError::Artifacts(format!("credentials: {e}")))?;

        let bucket = Bucket::new(bucket_name, region.clone(), credentials.clone())
            .map_err(|e| AutoForgeError::Artifacts(format!("bucket init: {e}")))?
            .with_path_style();

        let exists = bucket
            .exists()
            .await
            .map_err(|e| AutoForgeError::Artifacts(format!("connectivity check: {e}")))?;

        if exists {
            return Ok(bucket);
        }

        match Bucket::create_with_path_style(
            bucket_name,
            region,
            credentials,
            BucketConfiguration::default(),
        )
        .await
        {
            Ok(resp) if resp.response_code < 300 => Ok(resp.bucket.with_path_style()),
            Ok(resp) => {
                tracing::warn!(
                    code = resp.response_code,
                    body = %resp.response_text,
                    "bucket create returned non-2xx, assuming it already exists"
                );
                Ok(bucket)
            }
            Err(e) => {
                tracing::warn!(error = %e, "bucket create failed, will attempt to use existing bucket");
                Ok(bucket)
            }
        }
    }
}

#[async_trait]
impl ArtifactStore for S3ArtifactStore {
    async fn put(&self, key: &str, data: Bytes, content_type: &str) -> Result<ArtifactRef> {
        match &self.backend {
            Backend::S3(bucket) => {
                let resp = bucket
                    .put_object_with_content_type(format!("/{key}"), &data, content_type)
                    .await
                    .map_err(|e| AutoForgeError::Artifacts(format!("put {key}: {e}")))?;
                if resp.status_code() >= 300 {
                    return Err(AutoForgeError::Artifacts(format!(
                        "put {key} failed with status {}",
                        resp.status_code()
                    )));
                }
            }
            Backend::Memory(cache) => {
                cache.insert(key.to_string(), data);
            }
        }

        Ok(ArtifactRef {
            name: key.rsplit('/').next().unwrap_or(key).to_string(),
            key: key.to_string(),
            uri: self.uri_for(key),
            content_type: content_type.to_string(),
            sha256: None,
        })
    }

    async fn get(&self, key: &str) -> Result<Bytes> {
        match &self.backend {
            Backend::S3(bucket) => {
                let resp = bucket
                    .get_object(format!("/{key}"))
                    .await
                    .map_err(|e| AutoForgeError::Artifacts(format!("get {key}: {e}")))?;
                if resp.status_code() >= 300 {
                    return Err(AutoForgeError::Artifacts(format!(
                        "not found: {key} (status {})",
                        resp.status_code()
                    )));
                }
                Ok(resp.bytes().clone())
            }
            Backend::Memory(cache) => cache
                .get(key)
                .map(|v| v.clone())
                .ok_or_else(|| AutoForgeError::Artifacts(format!("not found: {key}"))),
        }
    }

    fn uri_for(&self, key: &str) -> String {
        format!("{}/{}/{}", self.endpoint, self.bucket_name, key)
    }

    fn is_durable(&self) -> bool {
        matches!(self.backend, Backend::S3(_))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn connect_falls_back_to_memory_when_unreachable_within_timeout() {
        let start = std::time::Instant::now();
        let store = S3ArtifactStore::connect(
            "http://127.0.0.1:19999",
            "test-bucket",
            Some("minioadmin"),
            Some("minioadmin"),
            "us-east-1",
        )
        .await;
        let elapsed = start.elapsed();
        assert!(
            elapsed < std::time::Duration::from_secs(15),
            "connect() took too long: {elapsed:?}"
        );
        assert!(!store.is_durable());
    }
}
