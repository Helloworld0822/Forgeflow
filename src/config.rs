use std::env;

#[derive(Debug, Clone)]
pub struct Config {
    pub host: String,
    pub port: u16,
    pub cursor_api_key: String,
    pub stitch_api_key: String,
    pub artifacts_endpoint: String,
    pub artifacts_bucket: String,
    pub default_repo_url: Option<String>,
}

impl Config {
    pub fn from_env() -> Self {
        Self {
            host: env::var("HOST").unwrap_or_else(|_| "0.0.0.0".into()),
            port: env::var("PORT")
                .ok()
                .and_then(|p| p.parse().ok())
                .unwrap_or(8080),
            cursor_api_key: env::var("CURSOR_API_KEY").unwrap_or_default(),
            stitch_api_key: env::var("STITCH_API_KEY").unwrap_or_default(),
            artifacts_endpoint: env::var("ARTIFACTS_ENDPOINT")
                .unwrap_or_else(|_| "http://localhost:9000".into()),
            artifacts_bucket: env::var("ARTIFACTS_BUCKET")
                .unwrap_or_else(|_| "autoforge".into()),
            default_repo_url: env::var("DEFAULT_REPO_URL").ok(),
        }
    }

    pub fn bind_addr(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }
}
