//! Onyx Node CLI — init, pulse, status.
//!
//! Usage:
//!   cargo run --bin onyx-node -- init --node-id onyx-hannover-01
//!   cargo run --bin onyx-node -- pulse --interval 30
//!   cargo run --bin onyx-node -- status

use anyhow::Result;
use clap::{Parser, Subcommand};
use onyx_node::{NodeIdentity, Pulse, PulseConfig};
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
        #[arg(long, default_value = "status")]
        status_dir: PathBuf,
    },
    /// Emit pulses on an interval. Writes status/last_pulse.json.
    Pulse {
        #[arg(long, default_value = "onyx-node-001")]
        node_id: String,
        #[arg(long, default_value_t = 30)]
        interval: u64,
        #[arg(long, default_value = "state/identity.json")]
        identity: PathBuf,
        #[arg(long, default_value = "status")]
        status_dir: PathBuf,
        #[arg(long, default_value = "nexus/mesh/v0")]
        topic: String,
    },
    /// Print identity and last pulse if present.
    Status {
        #[arg(long, default_value = "state/identity.json")]
        identity: PathBuf,
        #[arg(long, default_value = "status/last_pulse.json")]
        pulse: PathBuf,
    },
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
            status_dir,
        } => {
            let id = NodeIdentity::load_or_create(&identity, &node_id)?;
            info!("identity ready node_id={} independent_id={}", id.node_id, id.independent_id.as_str());
            let pulse = Pulse::alive(&id, "nexus/mesh/v0");
            let path = pulse.write_status(&status_dir)?;
            println!("{}", pulse.to_pretty_json()?);
            info!("pulse written {}", path.display());
        }
        Command::Pulse {
            node_id,
            interval,
            identity,
            status_dir,
            topic,
        } => {
            let id = NodeIdentity::load_or_create(&identity, &node_id)?;
            let cfg = PulseConfig {
                interval_secs: interval,
                status_dir: status_dir.clone(),
                topic: topic.clone(),
            };
            info!(
                "pulse loop node_id={} independent_id={} interval={}s",
                id.node_id,
                id.independent_id.as_str(),
                cfg.interval_secs
            );
            let mut tick = tokio::time::interval(Duration::from_secs(cfg.interval_secs));
            loop {
                tick.tick().await;
                let pulse = Pulse::alive(&id, &cfg.topic);
                let path = pulse.write_status(&cfg.status_dir)?;
                println!("── Onyx Pulse ──────────────────────────────");
                println!("{}", pulse.to_pretty_json()?);
                info!("pulse written {}", path.display());
            }
        }
        Command::Status { identity, pulse } => {
            if identity.exists() {
                println!("identity:\n{}", fs::read_to_string(&identity)?);
            } else {
                println!("identity: MISSING ({})", identity.display());
            }
            if pulse.exists() {
                println!("last pulse:\n{}", fs::read_to_string(&pulse)?);
            } else {
                println!("last pulse: MISSING ({})", pulse.display());
            }
        }
    }
    Ok(())
}
