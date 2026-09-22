# Runtime state (local only)

Files under `state/` are operator-local. They must not land on the public
`onyx-node` tree.

## Files

| Path | Purpose |
|------|--------|
| `state/identity.json` | `node_id` + `independent:<hex>` |
| `state/runtime.json` | transport enum + optional public fingerprint |

## Example `runtime.json`

```json
{
  "node_id": "onyx-hannover-01",
  "independent_id": "independent:0123abcd…",
  "transport": "tailscale",
  "fingerprint": "onyx-hannover-01.ts.net",
  "updated_at": "2026-09-22T21:50:00+00:00"
}
```

The private sister repository `onyx-hannover` may keep operator notes about
which machine owns this node. Keys stay off git entirely.
