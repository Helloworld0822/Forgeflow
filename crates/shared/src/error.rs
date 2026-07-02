use thiserror::Error;

#[derive(Debug, Error)]
pub enum AutoForgeError {
    #[error("cursor API error: {0}")]
    CursorApi(String),

    #[error("stitch API error: {0}")]
    StitchApi(String),

    #[error("ingest error: {0}")]
    Ingest(String),

    #[error("artifact store error: {0}")]
    Artifacts(String),

    #[error("orchestration error: {0}")]
    Orchestrator(String),

    #[error("stage {stage:?} failed: {message}")]
    StageFailed {
        stage: crate::StageId,
        message: String,
    },

    #[error("invalid state transition: {from:?} -> {to:?}")]
    InvalidTransition {
        from: crate::PipelineState,
        to: crate::PipelineState,
    },

    #[error("not found: {0}")]
    NotFound(String),
}

pub type Result<T> = std::result::Result<T, AutoForgeError>;
