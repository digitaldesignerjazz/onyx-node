//! Local runtime state. Overlay fingerprints live here, never in git.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

/// Allowed transports. Identity does not depend on these.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Transport {
    None,
    Tailscale,
    Netbird,
    Yggdrasil,
    Wireguard,
}

impl Transport {
    pub fn parse(raw: &str) -> Self {
        match raw.to_ascii_lowercase().as_str() {
            "tailscale" => Self::Tailscale,
            "netbird" => Self::Netbird,
            "yggdrasil" => Self::Yggdrasil,
            "wireguard" | "wg" => Self::Wireguard,
            _ => Self::None,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::None => "none",
            Self::Tailscale => "tailscale",
            Self::Netbird => "netbird",
            Self::Yggdrasil => "yggdrasil",
            Self::Wireguard => "wireguard",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeState {
    pub node_id: String,
    pub independent_id: String,
    pub transport: Transport,
    /// Public overlay hint only. Never an auth key.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fingerprint: Option<String>,
    pub updated_at: String,
}

impl RuntimeState {
    pub fn new(node_id: &str, independent_id: &str, transport: Transport) -> Self {
        Self {
            node_id: node_id.into(),
            independent_id: independent_id.into(),
            transport,
            fingerprint: None,
            updated_at: chrono::Utc::now().to_rfc3339(),
        }
    }

    pub fn with_fingerprint(mut self, fingerprint: Option<String>) -> Self {
        if let Some(fp) = fingerprint {
            let trimmed = fp.trim().to_string();
            if !trimmed.is_empty() {
                self.fingerprint = Some(trimmed);
            }
        }
        self
    }

    pub fn load_or_create(path: &Path, seed: Self) -> Result<Self> {
        if path.exists() {
            let raw = fs::read_to_string(path)
                .with_context(|| format!("read runtime {}", path.display()))?;
            return Ok(serde_json::from_str(&raw)?);
        }
        seed.save(path)?;
        Ok(seed)
    }

    pub fn save(&self, path: &Path) -> Result<()> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(path, serde_json::to_string_pretty(self)?)?;
        Ok(())
    }
}

/// Refuse values that look like secrets before they reach the pulse extra.
pub fn looks_like_secret(value: &str) -> bool {
    let lower = value.to_ascii_lowercase();
    lower.contains("tskey-")
        || lower.contains("nbkey-")
        || lower.contains("setupkey")
        || lower.contains("authkey")
        || lower.contains("private")
        || value.split_whitespace().any(|w| w.len() > 64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_tailscale_auth_shape() {
        assert!(looks_like_secret("tskey-auth-EXAMPLEONLY"));
        assert!(!looks_like_secret("100.x.y.z"));
    }

    #[test]
    fn parses_wireguard_alias() {
        assert_eq!(Transport::parse("wireguard"), Transport::Wireguard);
        assert_eq!(Transport::parse("WG"), Transport::Wireguard);
        assert_eq!(Transport::Wireguard.as_str(), "wireguard");
    }
}
