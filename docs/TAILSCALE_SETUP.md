# Tailscale integration (preferred overlay)

Public guide for recording **Tailscale** as the Onyx dataplane.

**No auth keys, no `tskey-` values, no tailnet secrets in this file.**

Tailscale is the preferred transport when the binary is present.
It does not replace local identity. `independent:<hex>` still lives in
`state/identity.json` if the tailnet is down.

Alternatives:

- [Yggdrasil](YGGDRASIL_SETUP.md) — public mesh
- [WireGuard](WIREGUARD_SETUP.md) — private tunnel you operate yourself

Onyx records **one** transport label. The others may still run on the host.

---

## Why Tailscale

| | Tailscale | Yggdrasil | WireGuard |
|---|---|---|---|
| Model | Tailnet you join | Public mesh | Tunnel you configure |
| Addressing | `100.x.y.z` plus MagicDNS | Overlay IPv6 | Addresses you assign |
| Keys in Onyx | Never. CLI refuses `tskey-` | Never | Never |
| Role here | Preferred dataplane | Companion mesh | Private alternative |

---

## 1. Install

Debian / Ubuntu:

```bash
curl -fsSL https://tailscale.com/install.sh | sh
tailscale version
```

The install script is Tailscale's. Read it before you pipe it to a shell
if you do not already trust that host.

## 2. Join the tailnet (auth stays outside git)

Interactive, on a machine with a browser:

```bash
sudo tailscale up
```

Headless, with a **one-time** auth key from the Tailscale admin console.
Do not paste the key into this repo, into chat, or into `--fingerprint`:

```bash
read -rs TS_AUTH_KEY
sudo tailscale up --auth-key "$TS_AUTH_KEY" --hostname onyx-hannover-01
unset TS_AUTH_KEY
```

`tailscale up` may also take `--ssh` or exit-node flags. Those are operator
choices and are not required for Onyx identity or pulse.

## 3. Check the node

```bash
tailscale status
tailscale ip -4
```

You want:

- this host listed as online
- a `100.` address
- a MagicDNS name such as `onyx-hannover-01.your-tailnet.ts.net`

The other Nexus host must be in the **same** tailnet (or a shared node).
Onyx does not create that tailnet and does not provision `*.grok.me`.

## 4. Point Onyx at Tailscale

The fingerprint is a **public hint**: MagicDNS name or the `100.` address.
Never an auth key.

```bash
cp config/onyx.tailscale.toml.example config/onyx.toml
```

Or flags:

```bash
cargo run --bin onyx-node -- init --node-id onyx-hannover-01 \
  --transport tailscale --fingerprint onyx-hannover-01.ts.net
```

The CLI refuses fingerprints that contain `tskey-`, `authkey`, `private`,
or a single token longer than 64 characters.

Pulse loop, same label:

```bash
cargo run --bin onyx-node -- pulse --interval 30 \
  --transport tailscale --fingerprint onyx-hannover-01.ts.net
```

What gets written (gitignored):

| Path | Contents |
|------|----------|
| `state/runtime.json` | `"transport": "tailscale"` plus the public fingerprint |
| `status/last_pulse.json` | local pulse |
| `status/last_mesh_envelope.json` | heartbeat; transport name only |

## 5. Reach another Nexus node

Once both hosts show in `tailscale status`, use the peer's MagicDNS name
or `100.` address. No extra peer URI, no WireGuard config on disk.

If status shows the peer offline, fix Tailscale first. Onyx will still
pulse with identity intact.

## Security notes

- Auth keys are single-use or reusable secrets. They never enter git.
- Reusable keys belong in the Tailscale admin console, not in `config/`.
- `--fingerprint` is a name or address, not a credential.
- Tailscale ACLs are the access policy. This repo does not ship ACL JSON
  because a real ACL names your tailnet.

---

Operator: Esslinger & Co. · Nexus Initiative
