//! nxmesh publish hook.
//!
//! Default build writes the envelope to disk.
//! Enable live gossip with:
//!
//! ```toml
//! nxmesh = { git = "https://github.com/digitaldesignerjazz/nexus", branch = "main" }
//! ```
//!
//! and `--features nxmesh`.

use crate::pulse::Pulse;
use anyhow::Result;
use tracing::{info, warn};

pub struct PublishReport {
    pub mode: &'static str,
    pub bytes: usize,
}

/// Publish or record the heartbeat envelope.
/// Live path is compiled only with feature `nxmesh`.
pub fn publish_pulse(pulse: &Pulse) -> Result<PublishReport> {
    let bytes = pulse.to_mesh_json()?;

    #[cfg(feature = "nxmesh")]
    {
        // Placeholder until the nxmesh crate exposes a stable publish API
        // matching York Autotype. Do not invent a live client here.
        warn!("nxmesh feature enabled but live client is not wired in this seed");
        info!(len = bytes.len(), "envelope ready for nxmesh topic nexus/mesh/v0");
        return Ok(PublishReport {
            mode: "nxmesh-pending",
            bytes: bytes.len(),
        });
    }

    #[cfg(not(feature = "nxmesh"))]
    {
        info!(len = bytes.len(), "local envelope only — nxmesh feature off");
        let _ = bytes;
        Ok(PublishReport {
            mode: "local",
            bytes: pulse.to_mesh_json()?.len(),
        })
    }
}
