//! Wire-compatible copy of `nxmesh::protocol::MeshMessage` and nxmesh's
//! identity key format (libp2p protobuf-encoded Ed25519 keypair).
//!
//! Source of truth: digitaldesignerjazz/nexus `mesh/noise-quic`. Kept local
//! until that crate compiles; the JSON shape must stay identical.

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use libp2p::identity::Keypair;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "payload")]
pub enum MeshMessage {
    Gossip {
        from: String,
        text: String,
        ts: DateTime<Utc>,
    },
    AgentHeartbeat {
        agent: String,
        node_id: String,
        status: String,
        ts: DateTime<Utc>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        extra: Option<serde_json::Value>,
    },
    DataRequest {
        id: String,
        topic: String,
        query: serde_json::Value,
    },
    DataResponse {
        id: String,
        data: serde_json::Value,
    },
    PeerIntro {
        peer_id: String,
        addrs: Vec<String>,
    },
}

/// Same on-disk format as `nxmesh::NodeIdentity::load_or_generate`.
pub fn load_or_generate_key(path: &Path) -> Result<Keypair> {
    if path.exists() {
        let data = fs::read(path).context("read mesh key")?;
        return Keypair::from_protobuf_encoding(&data).context("invalid mesh key encoding");
    }
    let kp = Keypair::generate_ed25519();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, kp.to_protobuf_encoding().context("encode mesh key")?)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let _ = fs::set_permissions(path, fs::Permissions::from_mode(0o600));
    }
    Ok(kp)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn heartbeat_json_matches_nxmesh_envelope() {
        let m = MeshMessage::AgentHeartbeat {
            agent: "onyx-node".into(),
            node_id: "n".into(),
            status: "alive".into(),
            ts: Utc::now(),
            extra: None,
        };
        let v = serde_json::to_value(&m).unwrap();
        assert_eq!(v["type"], "AgentHeartbeat");
        assert_eq!(v["payload"]["node_id"], "n");
        assert!(v["payload"].get("extra").is_none());
    }

    #[test]
    fn key_roundtrip() {
        let p = std::env::temp_dir().join(format!("onyx-key-{}", rand::random::<u32>()));
        let a = load_or_generate_key(&p).unwrap();
        let b = load_or_generate_key(&p).unwrap();
        assert_eq!(a.public(), b.public());
        let _ = fs::remove_file(p);
    }
}
