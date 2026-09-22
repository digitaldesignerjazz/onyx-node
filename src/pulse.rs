use crate::identity::NodeIdentity;
use crate::runtime::RuntimeState;
use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::json;
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

/// Compatible with nxmesh::protocol::MeshMessage::AgentHeartbeat.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pulse {
    pub agent: String,
    pub node_id: String,
    pub independent_id: String,
    pub status: String,
    pub ts: DateTime<Utc>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extra: Option<serde_json::Value>,
}

impl Pulse {
    pub fn alive(identity: &NodeIdentity, topic: &str, runtime: Option<&RuntimeState>) -> Self {
        let transport = runtime
            .map(|r| r.transport.as_str())
            .unwrap_or("none");
        Self {
            agent: "onyx-node".into(),
            node_id: identity.node_id.clone(),
            independent_id: identity.independent_id.as_str().to_string(),
            status: "alive".into(),
            ts: Utc::now(),
            extra: Some(json!({
                "prototype": "Onyx Node",
                "version": env!("CARGO_PKG_VERSION"),
                "plane": identity.plane,
                "topic": topic,
                "transport": transport,
                "capabilities": ["identity", "pulse", "mesh-heartbeat"],
                "mesh_substrate": "nxmesh",
                "github": "https://github.com/digitaldesignerjazz/onyx-node"
            })),
        }
    }

    /// Tagged envelope expected by nxmesh Gossipsub.
    pub fn to_mesh_json(&self) -> serde_json::Result<Vec<u8>> {
        let envelope = json!({
            "type": "AgentHeartbeat",
            "payload": {
                "agent": self.agent,
                "node_id": self.node_id,
                "status": self.status,
                "ts": self.ts,
                "extra": self.extra
            }
        });
        serde_json::to_vec(&envelope)
    }

    pub fn to_pretty_json(&self) -> Result<String> {
        Ok(serde_json::to_string_pretty(self)?)
    }

    pub fn write_status(&self, status_dir: &std::path::Path) -> Result<PathBuf> {
        fs::create_dir_all(status_dir)?;
        let path = status_dir.join("last_pulse.json");
        fs::write(&path, self.to_pretty_json()?)?;
        let envelope_path = status_dir.join("last_mesh_envelope.json");
        let envelope = serde_json::from_slice::<serde_json::Value>(&self.to_mesh_json()?)?;
        fs::write(&envelope_path, serde_json::to_string_pretty(&envelope)?)?;
        Ok(path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::identity::NodeIdentity;

    #[test]
    fn pulse_envelope_matches_nxmesh() {
        let id = NodeIdentity::new("onyx-test-01");
        let p = Pulse::alive(&id, "nexus/mesh/v0", None);
        let bytes = p.to_mesh_json().unwrap();
        let v: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(v["type"], "AgentHeartbeat");
        assert_eq!(v["payload"]["agent"], "onyx-node");
        assert_eq!(v["payload"]["status"], "alive");
        assert!(v["payload"]["extra"].is_object());
        assert_eq!(v["payload"]["extra"]["topic"], "nexus/mesh/v0");
        assert_eq!(v["payload"]["extra"]["transport"], "none");
    }
}
