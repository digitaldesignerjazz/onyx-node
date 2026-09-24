# Onyx Node architecture (public)

Status: **Proposed** · seeded 2026-09-22 · pulse envelope aligned 2026-09-22

## Purpose

Onyx is the durable edge node of the Nexus independent mesh plane.
It answers three questions locally, without a vendor control plane:

1. Who am I? — `independent:<hex>`
2. Am I alive? — pulse file + nxmesh heartbeat envelope
3. How do I speak, if a transport exists? — optional overlay fingerprint in local runtime state

## Layers

```
+-------------------------------------------------------+
|  operator / swarm roles (not this repo)               |
+-------------------------------------------------------+
|  onyx-node CLI                                        |
|    init | pulse | status                              |
+-------------------------------------------------------+
|  identity  |  pulse  |  publish hook                  |
+-------------------------------------------------------+
|  independent mesh plane                               |
|  (identity + pulse live even if overlay is down)      |
+-------------------------------------------------------+
|  optional transport                                   |
|  Tailscale preferred · NetBird secondary · Yggdrasil  |
|  WireGuard — private tunnel alternative               |
|  companion overlay only — never identity replacement  |
+-------------------------------------------------------+
```

## Identity

- First `init` writes `state/identity.json`.
- `node_id` is an operator label (`onyx-hannover-01`).
- `independent_id` is `independent:` plus 32 hex chars.
- No private key is written in v0.1. Signing arrives in a later spec.

## Pulse and heartbeat

See [MESH_HEARTBEAT.md](MESH_HEARTBEAT.md). Local pulse plus tagged envelope
`{ "type": "AgentHeartbeat", "payload": { … } }` on topic `nexus/mesh/v0`.

## Transport policy

Record which fingerprint is live in `state/runtime.json` only:

- none — identity still valid
- tailscale — preferred dataplane when the binary exists
- netbird — secondary
- yggdrasil — public mesh companion; see [YGGDRASIL_SETUP.md](YGGDRASIL_SETUP.md)
- wireguard — private tunnel alternative; see [WIREGUARD_SETUP.md](WIREGUARD_SETUP.md)

Never log auth keys. The CLI refuses `tskey-` / `nbkey-` shaped flags.
WireGuard `PrivateKey` values stay on the host under `/etc/wireguard/`.

## Wizard Q

Onyx may later carry a dry-run caster hook. It does not settle, sign
or submit runes in v0.1. Spec remains Proposed.

## Start order (operator)

1. Resolve path root.
2. `onyx-node init --node-id onyx-hannover-01`
3. Optional: `--transport tailscale --fingerprint <public-hint>`
4. `onyx-node pulse`
5. Prototypes after pulse is green.
