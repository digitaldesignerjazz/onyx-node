# onyx-node listen

```bash
cargo run --bin onyx-node -- listen --topic nexus/mesh/v0 --interval 30 \
  [--peers /ip4/HOST/tcp/4710,/ip6/YGG/tcp/4710] [--listen-addr ...] [--status-dir status]
```

- Transport: libp2p gossipsub, signed + strict validation, TCP+Noise+Yamux and QUIC-v1,
  identify `/nexus/nxmesh/0.1.0`. Payloads are nxmesh `MeshMessage` JSON (`src/mesh_proto.rs`).
- Default bind: `/ip4/0.0.0.0/tcp/4710`, `/ip4/0.0.0.0/udp/4710/quic-v1`.
- Peers: `--peers` plus `config/peers.txt` and `state/peers.txt` (one multiaddr per line).
  Configured peers are re-dialed every interval. mDNS is on unless `--no-mdns`.
- Mesh key: `state/mesh.key` (libp2p protobuf Ed25519, same format as nxmesh, gitignored).

Outputs (all gitignored):

| File | Content |
|------|---------|
| `status/peers/<node_id>.json` | latest message per sender (`received_at`, `from_peer`, `message`) |
| `status/peers/pulses.log` | one JSON line per received message |
| `status/counter_pulses.log` | one JSON line per sent counter-pulse (`round`, `published`, peers) |
| `status/last_pulse.json` | own latest pulse |
| `status/listen_state.json` | peer id, listen addrs, connected peers |

Ops: `scripts/start-listen.sh`, `scripts/watchdog.sh` (idempotent restart, logs to
`status/watchdog.log`), `scripts/start-probe.sh` (local loopback **test** peer `onyx-probe`).

Needs rustc >= 1.88 for current libp2p deps (rustup stable).
