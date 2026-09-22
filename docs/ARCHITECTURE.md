# Onyx Node architecture (public)

Status: **Proposed** · seeded 2026-09-22

## Purpose

Onyx is the durable edge node of the Nexus independent mesh plane.
It answers three questions locally, without a vendor control plane:

1. Who am I? — `independent:<hex>`
2. Am I alive? — pulse file + heartbeat envelope
3. How do I speak, if a transport exists? — optional overlay fingerprint

## Layers

```
+-------------------------------------------------------+
|  operator / swarm roles (not this repo)               |
+-------------------------------------------------------+
|  onyx-node CLI                                        |
|    init | pulse | status                              |
+-------------------------------------------------------+
|  identity  |  pulse  |  heartbeat envelope            |
+-------------------------------------------------------+
|  independent mesh plane                               |
|  (identity + pulse live even if overlay is down)      |
+-------------------------------------------------------+
|  optional transport                                   |
|  Tailscale preferred · NetBird secondary · Yggdrasil  |
|  companion IPv6 only — never identity replacement     |
+-------------------------------------------------------+
```

## Identity

- First `init` writes `state/identity.json`.
- `node_id` is an operator label (`onyx-hannover-01`).
- `independent_id` is `independent:` plus 32 hex chars.
- No private key is written in v0.1. Signing arrives in a later spec.

## Pulse and heartbeat

Pulse is a local file. Heartbeat is the same payload shaped for `nxmesh`:

```json
{
  "type": "AgentHeartbeat",
  "agent": "onyx-node",
  "node_id": "onyx-hannover-01",
  "independent_id": "independent:00..ff",
  "status": "alive",
  "topic": "nexus/mesh/v0",
  "ts": "2026-09-22T21:00:00+00:00"
}
```

Compatible in spirit with `york-autotype` heartbeats. Live mesh publish
waits until `nxmesh` is linked as a path or git dependency.

## Transport policy

Record which fingerprint is live in local runtime state, never in this
public tree:

- none — identity still valid
- tailscale — preferred dataplane when the binary exists
- netbird — secondary
- yggdrasil — companion only

Never log auth keys.

## Wizard Q

Onyx may later carry a dry-run caster hook. It does not settle, sign
or submit runes in v0.1. Spec remains Proposed.

## Start order (operator)

1. Resolve path root.
2. `onyx-node init`
3. `onyx-node pulse`
4. Attach transport only if present.
5. Prototypes after pulse is green.
