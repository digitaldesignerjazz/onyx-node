//! Onyx Node CLI — init, pulse, status, listen.
//!
//! Usage:
//!   cargo run --bin onyx-node -- init --node-id onyx-hannover-01
//!   cargo run --bin onyx-node -- pulse --interval 30 --transport tailscale
//!   cargo run --bin onyx-node -- status
//!   cargo run --bin onyx-node -- listen --topic nexus/mesh/v0 --interval 30

use anyhow::{bail, Result};
use clap::{Parser, Subcommand};
use onyx_node::{
    looks_like_secret, publish_pulse, NodeIdentity, Pulse, PulseConfig, RuntimeState, Transport,
};
use std::fs;
use std::path::PathBuf;
use std::time::Duration;
use tracing::{info, Level};
use tracing_subscriber::EnvFilter;

#[derive(Parser, Debug)]
#[command(name = "onyx-node", about = "Onyx Node — independent-mesh edge pulse")]
struct Args {
    #[command(subcommand)]
    cmd: Command,
}

#[derive(Subcommand, Debug)]
enum Command {
    /// Create or load local independent identity and emit one pulse.
    Init {
        #[arg(long, default_value = "onyx-node-001")]
        node_id: String,
        #[arg(long, default_value = "state/identity.json")]
        identity: PathBuf,
        #[arg(long, default_value = "state/runtime.json")]
        runtime: PathBuf,
        #[arg(long, default_value = "status")]
        status_dir: PathBuf,
        #[arg(long, default_value = "none")]
        transport: String,
        /// Public overlay hint only. Auth keys are refused.
        #[arg(long)]
        fingerprint: Option<String>,
    },
    /// Emit pulses on an interval. Writes status/last_pulse.json.
    Pulse {
        #[arg(long, default_value = "onyx-node-001")]
        node_id: String,
        #[arg(long, default_value_t = 30)]
        interval: u64,
        #[arg(long, default_value = "state/identity.json")]
        identity: PathBuf,
        #[arg(long, default_value = "state/runtime.json")]
        runtime: PathBuf,
        #[arg(long, default_value = "status")]
        status_dir: PathBuf,
        #[arg(long, default_value = "nexus/mesh/v0")]
        topic: String,
        #[arg(long, default_value = "none")]
        transport: String,
        #[arg(long)]
        fingerprint: Option<String>,
    },
    /// Join the nxmesh topic, record incoming pulses to <status>/peers/ and
    /// send a counter-pulse every interval.
    #[cfg(feature = "listen")]
    Listen {
        #[arg(long, default_value = "onyx-node-001")]
        node_id: String,
        #[arg(long, default_value = "nexus/mesh/v0")]
        topic: String,
        #[arg(long, default_value_t = 30)]
        interval: u64,
        /// Comma-separated peer multiaddrs. Also read from config/peers.txt
        /// and state/peers.txt (one per line) when present.
        #[arg(long)]
        peers: Option<String>,
        /// Comma-separated bind multiaddrs.
        #[arg(long, default_value = onyx_node::listen::DEFAULT_LISTEN_ADDRS)]
        listen_addr: String,
        #[arg(long, default_value = "status")]
        status_dir: PathBuf,
        #[arg(long, default_value = "state/identity.json")]
        identity: PathBuf,
        #[arg(long, default_value = "state/runtime.json")]
        runtime: PathBuf,
        /// libp2p Ed25519 key (created if missing, gitignored).
        #[arg(long, default_value = "state/mesh.key")]
        mesh_key: PathBuf,
        /// Extra peer files to read (comma-separated).
        #[arg(long, default_value = "config/peers.txt,state/peers.txt")]
        peers_file: String,
        #[arg(long)]
        no_mdns: bool,
        /// Label this node's pulses as a local loopback test peer.
        #[arg(long)]
        test_peer: bool,
        #[arg(long, default_value = "none")]
        transport: String,
        #[arg(long)]
        fingerprint: Option<String>,
    },
    /// Print identity, runtime and last pulse if present.
    Status {
        #[arg(long, default_value = "state/identity.json")]
        identity: PathBuf,
        #[arg(long, default_value = "state/runtime.json")]
        runtime: PathBuf,
        #[arg(long, default_value = "status/last_pulse.json")]
        pulse: PathBuf,
    },
}

fn load_runtime(
    path: &PathBuf,
    node_id: &str,
    independent_id: &str,
    transport: &str,
    fingerprint: Option<String>,
) -> Result<RuntimeState> {
    if let Some(fp) = fingerprint.as_ref() {
        if looks_like_secret(fp) {
            bail!("fingerprint looks like a secret — refused. store keys outside git and outside this flag");
        }
    }
    let seed = RuntimeState::new(node_id, independent_id, Transport::parse(transport))
        .with_fingerprint(fingerprint);
    RuntimeState::load_or_create(path, seed)
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::from_default_env()
                .add_directive("onyx_node=info".parse()?)
                .add_directive(Level::INFO.into()),
        )
        .init();

    match Args::parse().cmd {
        Command::Init {
            node_id,
            identity,
            runtime,
            status_dir,
            transport,
            fingerprint,
        } => {
            let id = NodeIdentity::load_or_create(&identity, &node_id)?;
            let rt = load_runtime(
                &runtime,
                &id.node_id,
                id.independent_id.as_str(),
                &transport,
                fingerprint,
            )?;
            info!(
                "identity ready node_id={} independent_id={} transport={}",
                id.node_id,
                id.independent_id.as_str(),
                rt.transport.as_str()
            );
            let pulse = Pulse::alive(&id, "nexus/mesh/v0", Some(&rt));
            let path = pulse.write_status(&status_dir)?;
            let report = publish_pulse(&pulse)?;
            println!("{}", pulse.to_pretty_json()?);
            info!("pulse written {} publish={}", path.display(), report.mode);
        }
        Command::Pulse {
            node_id,
            interval,
            identity,
            runtime,
            status_dir,
            topic,
            transport,
            fingerprint,
        } => {
            let id = NodeIdentity::load_or_create(&identity, &node_id)?;
            let rt = load_runtime(
                &runtime,
                &id.node_id,
                id.independent_id.as_str(),
                &transport,
                fingerprint,
            )?;
            let cfg = PulseConfig {
                interval_secs: interval,
                status_dir: status_dir.clone(),
                topic: topic.clone(),
            };
            info!(
                "pulse loop node_id={} independent_id={} transport={} interval={}s",
                id.node_id,
                id.independent_id.as_str(),
                rt.transport.as_str(),
                cfg.interval_secs
            );
            let mut tick = tokio::time::interval(Duration::from_secs(cfg.interval_secs));
            loop {
                tick.tick().await;
                let pulse = Pulse::alive(&id, &cfg.topic, Some(&rt));
                let path = pulse.write_status(&cfg.status_dir)?;
                let report = publish_pulse(&pulse)?;
                println!("── Onyx Pulse ──────────────────────────────");
                println!("{}", pulse.to_pretty_json()?);
                info!("pulse written {} publish={}", path.display(), report.mode);
            }
        }
        #[cfg(feature = "listen")]
        Command::Listen {
            node_id,
            topic,
            interval,
            peers,
            listen_addr,
            status_dir,
            identity,
            runtime,
            mesh_key,
            peers_file,
            no_mdns,
            test_peer,
            transport,
            fingerprint,
        } => {
            use onyx_node::listen::{parse_peer_list, read_peer_file, ListenConfig};
            let id = NodeIdentity::load_or_create(&identity, &node_id)?;
            let rt = load_runtime(
                &runtime,
                &id.node_id,
                id.independent_id.as_str(),
                &transport,
                fingerprint,
            )?;
            let mut all_peers = peers.map(|p| parse_peer_list(&p)).unwrap_or_default();
            for f in parse_peer_list(&peers_file) {
                let from_file = read_peer_file(std::path::Path::new(&f));
                if !from_file.is_empty() {
                    info!("{} peer(s) from {}", from_file.len(), f);
                }
                all_peers.extend(from_file);
            }
            all_peers.dedup();
            let cfg = ListenConfig {
                topic,
                interval_secs: interval,
                peers: all_peers,
                listen_addrs: parse_peer_list(&listen_addr),
                status_dir,
                mesh_key,
                enable_mdns: !no_mdns,
                test_peer,
            };
            onyx_node::listen::run(cfg, id, rt).await?;
        }
        Command::Status {
            identity,
            runtime,
            pulse,
        } => {
            for (label, path) in [
                ("identity", identity),
                ("runtime", runtime),
                ("last pulse", pulse),
            ] {
                if path.exists() {
                    println!("{}:\n{}\n", label, fs::read_to_string(&path)?);
                } else {
                    println!("{}: MISSING ({})\n", label, path.display());
                }
            }
        }
    }
    Ok(())
}
