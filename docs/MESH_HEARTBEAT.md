# Onyx Node ↔ nxmesh AgentHeartbeat

## Envelope

Same tagged shape as York Autotype / `nxmesh::MeshMessage::AgentHeartbeat`:

```json
{
  "type": "AgentHeartbeat",
  "payload": {
    "agent": "onyx-node",
    "node_id": "onyx-hannover-01",
    "status": "alive",
    "ts": "2026-09-22T21:50:00Z",
    "extra": {
      "prototype": "Onyx Node",
      "version": "0.1.1",
      "plane": "independent",
      "topic": "nexus/mesh/v0",
      "transport": "none",
      "capabilities": ["identity", "pulse", "mesh-heartbeat"],
      "mesh_substrate": "nxmesh",
      "github": "https://github.com/digitaldesignerjazz/onyx-node"
    }
  }
}
```

`independent_id` lives on the local pulse file. It is **not** copied into the
public mesh payload by default, so a gossip frame does not leak the full
local identity string until a later spec says so.

## Status

| Layer | Status |
|-------|--------|
| Message shape | Compatible |
| Local pulse file | `status/last_pulse.json` |
| Mesh envelope file | `status/last_mesh_envelope.json` |
| GitHub Actions pulse | Every 6 hours |
| Live nxmesh publish | Feature flag `nxmesh` — crate not wired |

## Enable live publish later

1. Uncomment the `nxmesh` dependency in `Cargo.toml`.
2. Build with `--features nxmesh`.
3. Replace the pending branch in `src/publish.rs` with `NxMeshNode::publish`.

Until then `publish_pulse` records the envelope locally and returns mode `local`.

## Transport fingerprint

`--transport tailscale|netbird|yggdrasil|none` and optional `--fingerprint`
write **only** to `state/runtime.json` (gitignored).

Auth-shaped values (`tskey-`, `nbkey-`, long tokens) are refused.
Record the public hint (node name or overlay address), never the key.
