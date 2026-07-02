use crate::domain::ArtifactRef;
use crate::error::{AutoForgeError, Result};
use sha2::{Digest, Sha256};

#[derive(Debug, Clone)]
pub struct IngestResult {
    pub raw_text: String,
    pub page_count: u32,
    pub sha256: String,
}

pub fn ingest_pdf(bytes: &[u8]) -> Result<IngestResult> {
    let sha256 = hex::encode(Sha256::digest(bytes));

    let doc = lopdf::Document::load_mem(bytes)
        .map_err(|e| AutoForgeError::Ingest(format!("PDF parse failed: {e}")))?;

    let page_count = doc.get_pages().len() as u32;
    let raw_text = doc
        .extract_text(&[])
        .map_err(|e| AutoForgeError::Ingest(format!("text extraction failed: {e}")))?;

    if raw_text.trim().is_empty() {
        return Err(AutoForgeError::Ingest(
            "no extractable text — OCR fallback required".into(),
        ));
    }

    Ok(IngestResult {
        raw_text,
        page_count,
        sha256,
    })
}

pub fn to_artifacts(result: &IngestResult, base_uri: &str) -> Vec<ArtifactRef> {
    vec![
        ArtifactRef {
            name: "raw_text.md".into(),
            uri: format!("{base_uri}/raw_text.md"),
            content_type: "text/markdown".into(),
            sha256: Some(result.sha256.clone()),
        },
        ArtifactRef {
            name: "ingest_meta.json".into(),
            uri: format!("{base_uri}/ingest_meta.json"),
            content_type: "application/json".into(),
            sha256: None,
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_empty_bytes() {
        assert!(ingest_pdf(&[]).is_err());
    }
}
