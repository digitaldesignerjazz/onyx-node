use crate::identity::NodeIdentity;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct PulseConfig {
    pub interval_secs: u64,
    pub status_dir: PathBuf,
    pub topic: String,
}

impl Default for PulseConfig {
    fn default() -> Self {
        Self {
            interval_secs: 30,
            status_dir: PathBuf::from("status"),
            topic: "nexus/mesh/v0".into(),
        }
    }
}

/// Heartbeat envelope. Compatible in spirit with york-autotype / nxmesh.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pulse {
    #[serde(rename = "type")]
    pub kind: String,
    pub agent: String,
    pub node_id: String,
    pub independent_id: String,
    pub status: String,
    pub topic: String,
    pub ts: String,
}

impl Pulse {
    pub fn alive(identity: &NodeIdentity, topic: &str) -> Self {
        Self {
            kind: "AgentHeartbeat".into(),
            agent: "onyx-node".into(),
            node_id: identity.node_id.clone(),
            independent_id: identity.independent_id.as_str().to_string(),
            status: "alive".into(),
            topic: topic.into(),
            ts: chrono::Utc::now().to_rfc3339(),
        }
    }

    pub fn to_pretty_json(&self) -> Result<String> {
        Ok(serde_json::to_string_pretty(self)?)
    }

    pub fn write_status(&self, status_dir: &std::path::Path) -> Result<PathBuf> {
        fs::create_dir_all(status_dir)?;
        let path = status_dir.join("last_pulse.json");
        fs::write(&path, self.to_pretty_json()?)?;
        Ok(path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::identity::NodeIdentity;

    #[test]
    fn pulse_is_heartbeat() {
        let id = NodeIdentity::new("onyx-test-01");
        let p = Pulse::alive(&id, "nexus/mesh/v0");
        assert_eq!(p.kind, "AgentHeartbeat");
        assert_eq!(p.agent, "onyx-node");
        assert_eq!(p.status, "alive");
    }
}
