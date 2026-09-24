# Changelog

## 0.2.0-alpha.1 — 2026-09-24

Alpha. Not yet verified against a real remote mesh peer.

### Added
- `onyx-node listen` subcommand (`--topic`, `--interval`, `--peers`, `--listen-addr`,
  `--status-dir`, `--mesh-key`, `--peers-file`, `--no-mdns`, `--test-peer`).
- Live transport: libp2p gossipsub (signed, strict) over TCP+Noise+Yamux and QUIC-v1 on
  topic `nexus/mesh/v0`. nxmesh-compatible identify protocol and `MeshMessage` JSON.
- Incoming pulses go to `status/peers/<node_id>.json` (latest) and `status/peers/pulses.log`.
- Counter-pulse every interval, logged to `status/counter_pulses.log` with a round number,
  and `status/last_pulse.json` is refreshed. Persistent libp2p key in `state/mesh.key`.
- Peers from `--peers`, `config/peers.txt` and `state/peers.txt`, re-dialed every interval.
- `scripts/start-listen.sh`, `scripts/watchdog.sh` (idempotent restart) and
  `scripts/start-probe.sh` (local loopback test peer).
- Yggdrasil IPv6 bind support through `state/listen.args`. The Hannover node listens on
  `/ip6/200:47dd:ce9e:2bc8:9a79:9a43:fa20:7079/tcp/4710`. See docs/LISTEN.md.
- Debian package metadata (cargo-deb).

### Notes
- Needs rustc >= 1.88 (current libp2p dependencies).
- `src/mesh_proto.rs` holds a copy of nxmesh's `MeshMessage` until the nxmesh fix
  (nexus branch `fix/nxmesh-gossipsub-err`) lands on main.

## 0.1.1
- Seed: `init`, `pulse`, `status`; local pulse files; GitHub Actions pulse.
