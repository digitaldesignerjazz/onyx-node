//! `onyx-node listen` — live nxmesh listener + counter-pulse loop.
//!
//! Wire-compatible with `nxmesh::NxMeshNode`: libp2p gossipsub (signed,
//! strict validation, same message-id function) over TCP+Noise+Yamux and
//! QUIC, identify protocol `/nexus/nxmesh/0.1.0`, payloads are
//! `nxmesh::MeshMessage` JSON. A local swarm is built here because
//! `NxMeshNode::run(self)` consumes the node and so cannot publish while
//! its event loop is running.
//!
//! Every received gossip message is written to
//! `<status>/peers/<node_id|peer_id>.json` (latest) and appended to
//! `<status>/peers/pulses.log`. Every interval a counter-pulse
//! (`Pulse::alive`) is published, `<status>/last_pulse.json` is refreshed and
//! a line is appended to `<status>/counter_pulses.log`.

use crate::identity::NodeIdentity;
use crate::pulse::Pulse;
use crate::runtime::RuntimeState;
use anyhow::{Context, Result};
use futures::StreamExt;
use libp2p::swarm::behaviour::toggle::Toggle;
use libp2p::swarm::{NetworkBehaviour, SwarmEvent};
use libp2p::{gossipsub, identify, mdns, noise, ping, tcp, yamux, Multiaddr, PeerId};
use crate::mesh_proto::{load_or_generate_key, MeshMessage};
use serde_json::{json, Value};
use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::fs::{self, OpenOptions};
use std::hash::{Hash, Hasher};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::Duration;
use tracing::{info, warn};

pub const DEFAULT_LISTEN_ADDRS: &str = "/ip4/0.0.0.0/tcp/4710,/ip4/0.0.0.0/udp/4710/quic-v1";

#[derive(Debug, Clone)]
pub struct ListenConfig {
    pub topic: String,
    pub interval_secs: u64,
    pub peers: Vec<String>,
    pub listen_addrs: Vec<String>,
    pub status_dir: PathBuf,
    pub mesh_key: PathBuf,
    pub enable_mdns: bool,
    /// Marks this node's counter-pulses as a local loopback test peer.
    pub test_peer: bool,
}

#[derive(NetworkBehaviour)]
struct OnyxBehaviour {
    gossipsub: gossipsub::Behaviour,
    identify: identify::Behaviour,
    ping: ping::Behaviour,
    mdns: Toggle<mdns::tokio::Behaviour>,
}

/// Split a comma / newline list and drop comments and blanks.
pub fn parse_peer_list(raw: &str) -> Vec<String> {
    raw.split(|c| c == ',' || c == '\n')
        .map(|l| l.split('#').next().unwrap_or("").trim().to_string())
        .filter(|l| !l.is_empty())
        .collect()
}

/// Read peer multiaddrs from a file (one per line, `#` comments) if it exists.
pub fn read_peer_file(path: &Path) -> Vec<String> {
    fs::read_to_string(path)
        .map(|s| parse_peer_list(&s))
        .unwrap_or_default()
}

/// Safe filename for a peer record.
pub fn peer_file_name(key: &str) -> String {
    let cleaned: String = key
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() || c == '-' || c == '_' || c == '.' { c } else { '_' })
        .collect();
    let trimmed = cleaned.trim_matches('.');
    if trimmed.is_empty() { "unknown".into() } else { format!("{trimmed}.json") }
}

fn append_line(path: &Path, value: &Value) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let mut f = OpenOptions::new().create(true).append(true).open(path)?;
    writeln!(f, "{}", serde_json::to_string(value)?)?;
    Ok(())
}

fn heartbeat_from(pulse: &Pulse) -> MeshMessage {
    MeshMessage::AgentHeartbeat {
        agent: pulse.agent.clone(),
        node_id: pulse.node_id.clone(),
        status: pulse.status.clone(),
        ts: pulse.ts,
        extra: pulse.extra.clone(),
    }
}

/// Record one received gossip message. Returns the peer file written.
pub fn record_incoming(
    status_dir: &Path,
    data: &[u8],
    source: Option<String>,
    via: String,
    topic: &str,
) -> Result<(PathBuf, Value)> {
    let received_at = chrono::Utc::now();
    let (message_type, key, message) = match serde_json::from_slice::<MeshMessage>(data) {
        Ok(msg) => {
            let v = serde_json::to_value(&msg)?;
            let ty = v["type"].as_str().unwrap_or("unknown").to_string();
            let key = match &msg {
                MeshMessage::AgentHeartbeat { node_id, .. } => node_id.clone(),
                _ => source.clone().unwrap_or_else(|| via.clone()),
            };
            (ty, key, v)
        }
        Err(_) => (
            "unparsed".to_string(),
            source.clone().unwrap_or_else(|| via.clone()),
            json!({ "raw": String::from_utf8_lossy(data) }),
        ),
    };
    let record = json!({
        "received_at": received_at.to_rfc3339(),
        "topic": topic,
        "from_peer": source,
        "via_peer": via,
        "message_type": message_type,
        "message": message,
    });
    let peers_dir = status_dir.join("peers");
    fs::create_dir_all(&peers_dir)?;
    let path = peers_dir.join(peer_file_name(&key));
    fs::write(&path, serde_json::to_string_pretty(&record)?)?;
    append_line(&peers_dir.join("pulses.log"), &record)?;
    Ok((path, record))
}

/// Run the listener forever.
pub async fn run(cfg: ListenConfig, id: NodeIdentity, rt: RuntimeState) -> Result<()> {
    let keypair = load_or_generate_key(&cfg.mesh_key)
        .with_context(|| format!("mesh key {}", cfg.mesh_key.display()))?;
    let local_peer = PeerId::from(keypair.public());
    fs::create_dir_all(cfg.status_dir.join("peers"))?;

    let message_id_fn = |m: &gossipsub::Message| {
        let mut s = DefaultHasher::new();
        m.data.hash(&mut s);
        gossipsub::MessageId::from(s.finish().to_string())
    };
    let gs_cfg = gossipsub::ConfigBuilder::default()
        .heartbeat_interval(Duration::from_secs(10))
        .validation_mode(gossipsub::ValidationMode::Strict)
        .message_id_fn(message_id_fn)
        .build()
        .map_err(|e| anyhow::anyhow!("gossipsub config: {e}"))?;
    let mut gossipsub = gossipsub::Behaviour::new(
        gossipsub::MessageAuthenticity::Signed(keypair.clone()),
        gs_cfg,
    )
    .map_err(|e| anyhow::anyhow!("gossipsub: {e}"))?;
    let topic = gossipsub::IdentTopic::new(cfg.topic.clone());
    gossipsub.subscribe(&topic)?;

    let mdns = if cfg.enable_mdns {
        match mdns::tokio::Behaviour::new(mdns::Config::default(), local_peer) {
            Ok(m) => Some(m),
            Err(e) => {
                warn!("mDNS disabled: {e}");
                None
            }
        }
    } else {
        None
    };

    let behaviour = OnyxBehaviour {
        gossipsub,
        identify: identify::Behaviour::new(identify::Config::new(
            "/nexus/nxmesh/0.1.0".into(),
            keypair.public(),
        )),
        ping: ping::Behaviour::new(ping::Config::new()),
        mdns: Toggle::from(mdns),
    };

    let mut swarm = libp2p::SwarmBuilder::with_existing_identity(keypair)
        .with_tokio()
        .with_tcp(tcp::Config::default(), noise::Config::new, yamux::Config::default)?
        .with_quic()
        .with_dns()?
        .with_behaviour(|_| behaviour)?
        .with_swarm_config(|c| c.with_idle_connection_timeout(Duration::from_secs(120)))
        .build();

    for a in &cfg.listen_addrs {
        let addr: Multiaddr = a.parse().with_context(|| format!("listen addr {a}"))?;
        swarm.listen_on(addr)?;
    }

    let mut peer_addrs: Vec<Multiaddr> = Vec::new();
    for p in &cfg.peers {
        match p.parse::<Multiaddr>() {
            Ok(a) => peer_addrs.push(a),
            Err(e) => warn!("ignoring bad peer multiaddr {p}: {e}"),
        }
    }
    // dialed address -> connected peer
    let mut dialed: HashMap<Multiaddr, PeerId> = HashMap::new();
    for a in &peer_addrs {
        info!("dialing configured peer {a}");
        if let Err(e) = swarm.dial(a.clone()) {
            warn!("dial {a}: {e}");
        }
    }

    println!(
        "[listen] node_id={} peer_id={} topic={} interval={}s peers={:?} mdns={} test_peer={}",
        id.node_id, local_peer, cfg.topic, cfg.interval_secs, cfg.peers, cfg.enable_mdns, cfg.test_peer
    );
    if cfg.peers.is_empty() {
        println!("[listen] no configured peers — waiting for inbound dials / mDNS");
    }

    let period = Duration::from_secs(cfg.interval_secs.max(1));
    let mut tick = tokio::time::interval_at(tokio::time::Instant::now() + period, period);
    let mut round: u64 = 0;
    let mut listen_addrs: Vec<String> = Vec::new();

    loop {
        tokio::select! {
            ev = swarm.select_next_some() => match ev {
                SwarmEvent::NewListenAddr { address, .. } => {
                    println!("[listen] listening on {address}/p2p/{local_peer}");
                    listen_addrs.push(address.to_string());
                }
                SwarmEvent::ConnectionEstablished { peer_id, endpoint, num_established, .. } => {
                    if endpoint.is_dialer() {
                        let remote = endpoint.get_remote_address().clone();
                        for a in &peer_addrs {
                            if *a == remote || a.iter().zip(remote.iter()).all(|(x, y)| x == y) {
                                dialed.insert(a.clone(), peer_id);
                            }
                        }
                    }
                    println!("[listen] connected {peer_id} via {} (conns={num_established})", endpoint.get_remote_address());
                }
                SwarmEvent::ConnectionClosed { peer_id, num_established, cause, .. } => {
                    if num_established == 0 {
                        dialed.retain(|_, p| *p != peer_id);
                    }
                    println!("[listen] disconnected {peer_id} (remaining={num_established}) cause={cause:?}");
                }
                SwarmEvent::OutgoingConnectionError { peer_id, error, .. } => {
                    println!("[listen] outgoing connection error peer={peer_id:?}: {error}");
                }
                SwarmEvent::IncomingConnectionError { send_back_addr, error, .. } => {
                    println!("[listen] incoming connection error from {send_back_addr}: {error}");
                }
                SwarmEvent::Behaviour(OnyxBehaviourEvent::Mdns(mdns::Event::Discovered(list))) => {
                    for (peer, addr) in list {
                        println!("[listen] mDNS discovered {peer} at {addr}");
                        let _ = swarm.dial(addr);
                    }
                }
                SwarmEvent::Behaviour(OnyxBehaviourEvent::Gossipsub(gossipsub::Event::Subscribed { peer_id, topic })) => {
                    println!("[listen] peer {peer_id} subscribed to {topic}");
                }
                SwarmEvent::Behaviour(OnyxBehaviourEvent::Gossipsub(gossipsub::Event::Message {
                    propagation_source, message, ..
                })) => {
                    let src = message.source.map(|p| p.to_string());
                    match record_incoming(&cfg.status_dir, &message.data, src, propagation_source.to_string(), message.topic.as_str()) {
                        Ok((path, rec)) => println!(
                            "[pulse-in] {} type={} from={} node_id={} ts={} -> {}",
                            rec["received_at"].as_str().unwrap_or(""),
                            rec["message_type"].as_str().unwrap_or(""),
                            rec["from_peer"].as_str().unwrap_or("?"),
                            rec["message"]["payload"]["node_id"].as_str().unwrap_or("-"),
                            rec["message"]["payload"]["ts"].as_str().unwrap_or("-"),
                            path.display()
                        ),
                        Err(e) => warn!("failed to record incoming pulse: {e}"),
                    }
                }
                _ => {}
            },
            _ = tick.tick() => {
                round += 1;
                // re-dial configured peers that are not connected
                for a in &peer_addrs {
                    let up = dialed.get(a).map(|p| swarm.is_connected(p)).unwrap_or(false);
                    if !up {
                        let _ = swarm.dial(a.clone());
                    }
                }
                let mut pulse = Pulse::alive(&id, &cfg.topic, Some(&rt));
                if let Some(Value::Object(extra)) = pulse.extra.as_mut() {
                    extra.insert("round".into(), json!(round));
                    extra.insert("peer_id".into(), json!(local_peer.to_string()));
                    extra.insert("mesh_transport".into(), json!("nxmesh gossipsub (tcp+noise+yamux, quic-v1)"));
                    extra.insert("kind".into(), json!("counter-pulse"));
                    if cfg.test_peer {
                        extra.insert("test_peer".into(), json!("local loopback test peer — not a real mesh peer"));
                    }
                }
                if let Err(e) = pulse.write_status(&cfg.status_dir) {
                    warn!("write last_pulse.json: {e}");
                }
                let connected = swarm.connected_peers().count();
                let topic_peers = swarm.behaviour().gossipsub.all_peers()
                    .filter(|(_, t)| t.iter().any(|h| **h == topic.hash())).count();
                let data = serde_json::to_vec(&heartbeat_from(&pulse))?;
                let result = swarm.behaviour_mut().gossipsub.publish(topic.clone(), data);
                let (published, error) = match &result {
                    Ok(_) => (true, None),
                    Err(e) => (false, Some(format!("{e:?}"))),
                };
                let line = json!({
                    "round": round,
                    "ts": pulse.ts.to_rfc3339(),
                    "node_id": id.node_id,
                    "peer_id": local_peer.to_string(),
                    "topic": cfg.topic,
                    "published": published,
                    "error": error,
                    "connected_peers": connected,
                    "topic_peers": topic_peers,
                    "test_peer": cfg.test_peer,
                });
                if let Err(e) = append_line(&cfg.status_dir.join("counter_pulses.log"), &line) {
                    warn!("append counter_pulses.log: {e}");
                }
                let state = json!({
                    "node_id": id.node_id,
                    "peer_id": local_peer.to_string(),
                    "topic": cfg.topic,
                    "listen_addrs": listen_addrs,
                    "configured_peers": cfg.peers,
                    "connected_peers": swarm.connected_peers().map(|p| p.to_string()).collect::<Vec<_>>(),
                    "last_round": round,
                    "updated_at": chrono::Utc::now().to_rfc3339(),
                });
                let _ = fs::write(cfg.status_dir.join("listen_state.json"), serde_json::to_string_pretty(&state)?);
                println!(
                    "[pulse-out] round={round} ts={} published={published} connected={connected} topic_peers={topic_peers}{}",
                    pulse.ts.to_rfc3339(),
                    error.map(|e| format!(" error={e}")).unwrap_or_default()
                );
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn peer_list_parsing() {
        let v = parse_peer_list("/ip4/1.2.3.4/tcp/1, /ip4/5.6.7.8/tcp/2\n# c\n\n/dns4/x/tcp/3 # tail");
        assert_eq!(v, vec!["/ip4/1.2.3.4/tcp/1", "/ip4/5.6.7.8/tcp/2", "/dns4/x/tcp/3"]);
    }

    #[test]
    fn peer_file_names_are_safe() {
        assert_eq!(peer_file_name("onyx-probe"), "onyx-probe.json");
        assert_eq!(peer_file_name("../etc/passwd"), "_etc_passwd.json");
        assert_eq!(peer_file_name(""), "unknown");
    }

    #[test]
    fn records_heartbeat_by_node_id() {
        let dir = std::env::temp_dir().join(format!("onyx-rec-{}", rand::random::<u32>()));
        let msg = MeshMessage::AgentHeartbeat {
            agent: "onyx-node".into(),
            node_id: "peer-x".into(),
            status: "alive".into(),
            ts: chrono::Utc::now(),
            extra: None,
        };
        let data = serde_json::to_vec(&msg).unwrap();
        let (path, rec) = record_incoming(&dir, &data, Some("12D3src".into()), "12D3via".into(), "nexus/mesh/v0").unwrap();
        assert!(path.ends_with("peers/peer-x.json"));
        assert_eq!(rec["message_type"], "AgentHeartbeat");
        let log = fs::read_to_string(dir.join("peers/pulses.log")).unwrap();
        assert_eq!(log.lines().count(), 1);
        let _ = fs::remove_dir_all(dir);
    }

    /// Two in-process listeners on loopback: B dials A, A must receive B's
    /// counter-pulse over the real libp2p transport.
    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn loopback_pair_exchanges_pulses() {
        let base = std::env::temp_dir().join(format!("onyx-pair-{}", rand::random::<u32>()));
        let port: u16 = 20000 + (rand::random::<u16>() % 20000);
        let mk = |name: &str, listen: String, peers: Vec<String>| {
            let dir = base.join(name);
            let cfg = ListenConfig {
                topic: "nexus/mesh/test".into(),
                interval_secs: 1,
                peers,
                listen_addrs: vec![listen],
                status_dir: dir.join("status"),
                mesh_key: dir.join("mesh.key"),
                enable_mdns: false,
                test_peer: true,
            };
            let id = NodeIdentity::new(name);
            let rt = RuntimeState::new(name, id.independent_id.as_str(), crate::Transport::None);
            (cfg, id, rt)
        };
        let (ca, ia, ra) = mk("node-a", format!("/ip4/127.0.0.1/tcp/{port}"), vec![]);
        let (cb, ib, rb) = mk("node-b", "/ip4/127.0.0.1/tcp/0".into(), vec![format!("/ip4/127.0.0.1/tcp/{port}")]);
        let target = ca.status_dir.join("peers").join("node-b.json");
        let a = tokio::spawn(run(ca, ia, ra));
        tokio::time::sleep(Duration::from_millis(300)).await;
        let b = tokio::spawn(run(cb, ib, rb));
        let mut ok = false;
        for _ in 0..60 {
            if target.exists() { ok = true; break; }
            tokio::time::sleep(Duration::from_millis(500)).await;
        }
        a.abort();
        b.abort();
        let _ = fs::remove_dir_all(&base);
        assert!(ok, "node-a never received node-b's pulse");
    }
}
