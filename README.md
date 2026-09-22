# Onyx Node

**Durable independent-mesh edge node · Nexus lineage**

[![ci](https://github.com/digitaldesignerjazz/onyx-node/actions/workflows/ci.yml/badge.svg)](https://github.com/digitaldesignerjazz/onyx-node/actions/workflows/ci.yml)
[![pulse](https://github.com/digitaldesignerjazz/onyx-node/actions/workflows/pulse.yml/badge.svg)](https://github.com/digitaldesignerjazz/onyx-node/actions/workflows/pulse.yml)

Onyx is a **stone node**: local identity first, optional transport second.
It belongs to the independent mesh plane (`NEXUS_MESH=independent`).
A live overlay fingerprint (Tailscale or NetBird) may be recorded locally.
Identity never waits on a vendor control plane.

> Status: **seeded** · September 2026  
> Spec: **Proposed** · not a live public chain  
> License: [Apache-2.0](LICENSE)  
> Operator: Esslinger & Co. · GitHub [`digitaldesignerjazz`](https://github.com/digitaldesignerjazz)

---

## What this node is

| Piece | Role |
|-------|------|
| Identity | Local `independent:<hex>` node id, generated on first start |
| Pulse | `status/last_pulse.json` |
| Heartbeat | Tagged nxmesh envelope on topic `nexus/mesh/v0` |
| Transport | Optional. Tailscale preferred when present. Recorded only in `state/` |
| Wizard Q | Dry-run only until a public spec says otherwise |

## What this node is not

- Not a vendor-managed peer.
- Not a live QNET / QCoin settler.
- Not a dump of private swarm state, skilllogin files, keys or wallets.

See [docs/PUBLIC_BOUNDARY.md](docs/PUBLIC_BOUNDARY.md).

---

## Quick start

```bash
cargo run --bin onyx-node -- init --node-id onyx-hannover-01
cargo run --bin onyx-node -- pulse --interval 30 --transport none
cargo run --bin onyx-node -- status
```

Optional public overlay hint (never an auth key):

```bash
cargo run --bin onyx-node -- init --node-id onyx-hannover-01 \
  --transport tailscale --fingerprint onyx-hannover-01.ts.net
```

First start writes gitignored files:

- `state/identity.json`
- `state/runtime.json`
- `status/last_pulse.json`
- `status/last_mesh_envelope.json`

Copy `config/onyx.example.toml` to `config/onyx.toml` for local overrides.

Hannover operator notes live in the **private** sister repo `onyx-hannover`.

---

## Siblings

- Field: [Aether](https://github.com/digitaldesignerjazz/Aether)
- Ecosystem (public): [nexus-ecosystem-oss](https://github.com/digitaldesignerjazz/nexus-ecosystem-oss)
- Hub: [nexus](https://github.com/digitaldesignerjazz/nexus)
- Heartbeat cousin: [york-autotype](https://github.com/digitaldesignerjazz/york-autotype)
- Chain / runes: [qnet](https://github.com/digitaldesignerjazz/qnet) — Proposed

---

## License

Apache-2.0  
Esslinger & Co. / Nexus Initiative
