# Yggdrasil Setup for Onyx Node

Public guide for connecting the Onyx node to the Yggdrasil mesh.
**No private keys, no secrets in this file.**

---

## 1. Check the service

```bash
systemctl status yggdrasil --no-pager
```

Expect: `active (running)`.

## 2. Fix socket permissions (once)

The control socket is group-owned by `yggdrasil`:

```bash
ls -la /var/run/yggdrasil/yggdrasil.sock
# srw-rw---- 1 root yggdrasil ...

sudo apt install -y util-linux-extra   # provides newgrp/sg
sudo usermod -aG yggdrasil "$USER"

# apply without full logout:
sg yggdrasil -c "yggdrasilctl getSelf"
```

## 3. Find a public peer

Yggdrasil has no bootstrap nodes — you must peer with someone.
Use the public peer list, prefer nearby regions (Germany/Europe):

https://publicpeers.neilalexander.dev

Pick a `tls://` entry with low latency. Avoid distant countries.
Two nearby peers is enough. Do not add the same node twice over TLS and QUIC.

## 4. Add the peer to the config

```bash
sudo nano /etc/yggdrasil/yggdrasil.conf
```

Or merge the Germany snippet without touching PrivateKey:

```bash
sudo python3 config/yggdrasil/apply-peers.py
```

Leave `Listen` empty if you only do outbound peering.

## 5. Optimize for Onyx (Docker host)

Default multicast `Regex: .*` opens TLS listeners on every Docker veth.
Pin the admin socket, name the TUN `ygg0`, hide build info:

```bash
sudo python3 config/yggdrasil/apply-optimize.py
```

What changes:

| Knob | Value | Why |
|------|-------|-----|
| AdminListen | `unix:///var/run/yggdrasil/yggdrasil.sock` | stops the fallback warning |
| MulticastInterfaces | `eth.*` only | no listeners on docker_gwbridge / veth* |
| IfName | `ygg0` | stable TUN name |
| IfMTU | `65535` | official Linux default |
| NodeInfoPrivacy | `true` | no OS/arch/version in the mesh |
| Listen | `[]` | outbound only |

PrivateKey and Peers stay as they are.

## 6. Reload and verify

```bash
sudo systemctl restart yggdrasil
sleep 2
sg yggdrasil -c "yggdrasilctl getSelf"
sg yggdrasil -c "yggdrasilctl getPeers"
ip -6 addr show ygg0
```

A connected peer shows an address and a remote `tls://` endpoint.
If the list stays empty, check the peer string, outbound TLS, and that
the peer is online.

## 7. Onyx node transport

Once Yggdrasil is up, point the Onyx config at it:

```toml
[mesh]
transport = "yggdrasil"
```

Copy `config/onyx.example.toml` to `config/onyx.toml` and adjust.

## Security notes

- Never commit `PrivateKey` from `yggdrasil.conf` or `~/private.key`.
- `GroupPassword` left empty = public connectivity. Set it only for a
  private sub-mesh.
- `AllowedPublicKeys` is not a firewall for open ports.
- Do not add distant public peers. Latency becomes your routing cost.

---

Operator: Esslinger & Co. · Nexus Initiative
