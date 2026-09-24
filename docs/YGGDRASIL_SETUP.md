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

## 4. Add the peer to the config

```bash
sudo nano /etc/yggdrasil/yggdrasil.conf
```

Set:

```hocon
Peers: [
  "tls://<IP>:<PORT>"
]
```

Leave `Listen` empty if you only do outbound peering. Multicast discovery
(`MulticastInterfaces` with `Beacon: true`, `Listen: true`) is already the
default and works alongside explicit peers.

## 5. Reload and verify

```bash
sudo systemctl reload yggdrasil
sg yggdrasil -c "yggdrasilctl getPeers"
```

A connected peer shows up with its public key and address. If the list stays
empty, check the peer string, firewall (port must be reachable), and that the
peer is actually online.

## 6. Onyx node transport

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

---

Operator: Esslinger & Co. · Nexus Initiative
