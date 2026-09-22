use anyhow::{Context, Result};
use rand::RngCore;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

/// Local independent-plane identity. No private key in v0.1.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeIdentity {
    pub node_id: String,
    pub independent_id: IndependentId,
    pub created_at: String,
    pub plane: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct IndependentId(pub String);

impl IndependentId {
    pub fn generate() -> Self {
        let mut bytes = [0u8; 16];
        rand::thread_rng().fill_bytes(&mut bytes);
        Self(format!("independent:{}", hex::encode(bytes)))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl NodeIdentity {
    pub fn new(node_id: impl Into<String>) -> Self {
        Self {
            node_id: node_id.into(),
            independent_id: IndependentId::generate(),
            created_at: chrono::Utc::now().to_rfc3339(),
            plane: "independent".into(),
        }
    }

    pub fn load_or_create(path: &Path, node_id: &str) -> Result<Self> {
        if path.exists() {
            let raw = fs::read_to_string(path)
                .with_context(|| format!("read identity {}", path.display()))?;
            let id: Self = serde_json::from_str(&raw).context("parse identity.json")?;
            return Ok(id);
        }
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let id = Self::new(node_id);
        let pretty = serde_json::to_string_pretty(&id)?;
        fs::write(path, pretty)?;
        Ok(id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn independent_id_prefix() {
        let id = IndependentId::generate();
        assert!(id.as_str().starts_with("independent:"));
        assert_eq!(id.as_str().len(), "independent:".len() + 32);
    }
}
