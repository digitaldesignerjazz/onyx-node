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

## Hannover listener (Yggdrasil)

The Hannover box binds the listener on its Yggdrasil address as well as IPv4 by putting
the binds in `state/listen.args` (gitignored). `scripts/start-listen.sh` and the watchdog
read that file on every start:

```
--listen-addr /ip4/0.0.0.0/tcp/4710,/ip4/0.0.0.0/udp/4710/quic-v1,/ip6/200:47dd:ce9e:2bc8:9a79:9a43:fa20:7079/tcp/4710,/ip6/200:47dd:ce9e:2bc8:9a79:9a43:fa20:7079/udp/4710/quic-v1
```

Hannover multiaddrs. The PeerId comes from `state/mesh.key` and does not change across restarts:

```
/ip6/200:47dd:ce9e:2bc8:9a79:9a43:fa20:7079/tcp/4710/p2p/12D3KooWESUu46nLcMtAFHVLc5yFHv85YiGBQzQkZeWNwHBejQAy
/ip6/200:47dd:ce9e:2bc8:9a79:9a43:fa20:7079/udp/4710/quic-v1/p2p/12D3KooWESUu46nLcMtAFHVLc5yFHv85YiGBQzQkZeWNwHBejQAy
```

The Hannover Yggdrasil node has `Listen: []`, so it has no public peer URI to dial.
It keeps outbound TLS peerings to the public uplinks `ygg1/ygg2/ygg7.mk16.de:1338`.
Other nodes reach it through the Yggdrasil overlay. Peer your node with a public uplink
(the same mk16.de ones work) rather than with Hannover directly.

## Onyx WSL: join the Hannover listener

`sudo` asks for your password. Yggdrasil needs root for its TUN interface. Without a TUN
(`IfName: none`) you get no 200::/7 address and libp2p cannot dial it, so a user-space-only
install is not useful here.

```bash
# 0) Is Yggdrasil already installed and running?
sudo yggdrasilctl getSelf && sudo yggdrasilctl getPeers

# 1) Install Yggdrasil if missing (official Debian/Ubuntu repo)
sudo apt-get update && sudo apt-get install -y dirmngr gnupg curl git netcat-openbsd build-essential pkg-config
sudo mkdir -p /usr/local/apt-keys
gpg --fetch-keys https://neilalexander.s3.dualstack.eu-west-2.amazonaws.com/deb/key.txt
gpg --export 1C5162E133015D81A811239D1840CDAC6011C5EA | sudo tee /usr/local/apt-keys/yggdrasil-keyring.gpg > /dev/null
echo 'deb [signed-by=/usr/local/apt-keys/yggdrasil-keyring.gpg] http://neilalexander.s3.dualstack.eu-west-2.amazonaws.com/deb/ debian yggdrasil' | sudo tee /etc/apt/sources.list.d/yggdrasil.list
sudo apt-get update && sudo apt-get install -y yggdrasil
# the package writes /etc/yggdrasil.conf; the peer helper expects /etc/yggdrasil/yggdrasil.conf
[ -f /etc/yggdrasil/yggdrasil.conf ] || { sudo mkdir -p /etc/yggdrasil && sudo ln -sf /etc/yggdrasil.conf /etc/yggdrasil/yggdrasil.conf; }

# 2) Fresh checkout of the listen branch, so an existing ~/onyx-node identity stays untouched
git clone -b feat/listen https://github.com/digitaldesignerjazz/onyx-node.git ~/onyx-listen
cd ~/onyx-listen

# 3) Peer with the same public uplinks as Hannover (only the Peers block changes, PrivateKey is kept)
sudo python3 config/yggdrasil/apply-peers.py
sudo systemctl restart yggdrasil 2>/dev/null || { sudo pkill -x yggdrasil; sudo sh -c 'nohup yggdrasil -useconffile /etc/yggdrasil/yggdrasil.conf > /var/log/yggdrasil.log 2>&1 &'; }
sleep 5; sudo yggdrasilctl getPeers        # expect ygg1/ygg2.mk16.de "Up"

# 4) Can you reach Hannover over the overlay?
ping -6 -c3 200:47dd:ce9e:2bc8:9a79:9a43:fa20:7079
nc -6 -zv -w5 200:47dd:ce9e:2bc8:9a79:9a43:fa20:7079 4710

# 5) Rust (rustc >= 1.88 needed) + listener
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
. "$HOME/.cargo/env"
cargo run --bin onyx-node -- listen --topic nexus/mesh/v0 --interval 30 --node-id onyx-wsl \
  --peers /ip6/200:47dd:ce9e:2bc8:9a79:9a43:fa20:7079/tcp/4710/p2p/12D3KooWESUu46nLcMtAFHVLc5yFHv85YiGBQzQkZeWNwHBejQAy
```

Expect `[listen] connected 12D3KooWESUu…` followed by `[pulse-in] … node_id=onyx-node-001`
lines. Hannover writes Onyx's pulses to `status/peers/onyx-wsl.json`.
`--node-id` only takes effect when `state/identity.json` is first created, which is why step 2 uses a fresh clone.
