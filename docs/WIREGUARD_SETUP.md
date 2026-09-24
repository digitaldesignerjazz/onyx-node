# WireGuard as an alternative overlay

Public guide for using **WireGuard** instead of (or beside) Yggdrasil
on an Onyx node.

**No private keys, no shared secrets, no real endpoints in this file.**

Yggdrasil setup stays in [YGGDRASIL_SETUP.md](YGGDRASIL_SETUP.md).
WireGuard does not replace local identity. The independent id still
lives in `state/identity.json` even if the tunnel is down.

---

## Why WireGuard instead of Yggdrasil

| | Yggdrasil | WireGuard |
|---|---|---|
| Model | Public mesh, many peers | Private tunnel you control |
| Addressing | Overlay IPv6, self-assigned | Addresses you assign |
| Discovery | Multicast + explicit peers | Endpoints you configure |
| Keys | Node keypair inside Yggdrasil | `PrivateKey` / `PublicKey` per peer |
| Fit | Reach the wider mesh | Link Onyx to one Nexus host |

Use WireGuard when you want a closed path between Onyx and a Nexus
server you operate. Use Yggdrasil when you want the public mesh.
They can run at the same time; Onyx records **one** transport label.

---

## 1. Install

Debian / Ubuntu:

```bash
sudo apt update
sudo apt install -y wireguard-tools
```

Confirm:

```bash
wg --version
ip link show type wireguard || true
```

## 2. Generate keys locally (never commit them)

On **each** host (Onyx and the Nexus peer):

```bash
umask 077
wg genkey | tee /etc/wireguard/onyx.private | wg pubkey > /etc/wireguard/onyx.public
chmod 600 /etc/wireguard/onyx.private
```

`*.key` and `*.private` patterns are gitignored in this repo. Do not
paste the private key into chat, issues, or pulse fingerprints.

Exchange only the **public** key and a reachable endpoint (host:port).

## 3. Interface config (placeholders)

`/etc/wireguard/wg-nexus.conf` on Onyx. Replace every `<PLACEHOLDER>`.

```ini
[Interface]
# Address you assign on this node. Not an auth key.
Address = <ONYX_WG_ADDR>/32
PrivateKey = <ONYX_PRIVATE_KEY>
ListenPort = 51820

[Peer]
# Public key of the Nexus server. Safe to share with the peer only.
PublicKey = <NEXUS_PUBLIC_KEY>
# Optional: restrict what this peer may route to Onyx.
AllowedIPs = <NEXUS_WG_ADDR>/32
Endpoint = <NEXUS_HOST>:51820
PersistentKeepalive = 25
```

Mirror it on the Nexus host: swap keys, addresses, and set `Endpoint`
to Onyx only if Onyx has a stable public address. If Onyx is behind NAT,
leave Onyx's `Endpoint` pointing at the server and rely on keepalive.

## 4. Bring the tunnel up

```bash
sudo wg-quick up wg-nexus
sudo wg show
```

Enable at boot:

```bash
sudo systemctl enable --now wg-quick@wg-nexus
```

A live handshake shows `latest handshake` within two minutes.
No handshake means: wrong public key, firewall (UDP 51820), or endpoint.

## 5. Point Onyx at the tunnel

Copy the example and edit locally (gitignored):

```bash
cp config/onyx.wireguard.toml.example config/onyx.toml
```

Or pass flags. The fingerprint is a **public hint** (interface name or
tunnel address), never the private key:

```bash
cargo run --bin onyx-node -- init --node-id onyx-hannover-01 \
  --transport wireguard --fingerprint wg-nexus
```

The CLI refuses values that look like secrets (`private`, long tokens).

## 6. Compare with the Yggdrasil path

| Step | Yggdrasil | WireGuard |
|---|---|---|
| Daemon | `yggdrasil.service` | `wg-quick@wg-nexus` |
| Peers | public `tls://` URIs | one `[Peer]` you trust |
| Check | `yggdrasilctl getPeers` | `wg show` |
| Onyx label | `transport = "yggdrasil"` | `transport = "wireguard"` |

Identity and pulse do not wait on either overlay.

## Security notes

- Private keys stay in `/etc/wireguard/` with mode `0600`.
- Do not commit `wg-nexus.conf` if it contains `PrivateKey`.
- `AllowedIPs` is the route filter. Do not set `0.0.0.0/0` unless you
  intend this tunnel to be the default route.
- `nexusserver.grok.me` is not provisioned by this repo. Put a host you
  actually control in `Endpoint`.

---

Operator: Esslinger & Co. · Nexus Initiative
