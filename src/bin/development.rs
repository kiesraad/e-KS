use std::{io, path::PathBuf, process::Stdio};

use anyhow::{Context, Result};
use serde::Deserialize;
use tokio::{
    fs,
    process::{Child, Command},
    signal,
    signal::unix::{SignalKind, signal as unix_signal},
};

#[path = "../dev/utils.rs"]
mod utils;

use utils::{run, stop_running_containers, wait_for_postgres};

#[tokio::main]
async fn main() -> Result<()> {
    println!("⏳ Starting development environment...");

    stop_running_containers().await?;
    run("docker", &["compose", "up", "-d"]).await?;

    let config = load_config().await?;

    let mut children = Vec::new();
    let mut waited_for_postgres = false;

    for config in config.children {
        if config.wait_for_postgres && !waited_for_postgres {
            wait_for_postgres().await?;
            waited_for_postgres = true;
        }

        println!("🚀 Starting {}", config.name);
        children.push(ManagedChild {
            child: config.spawn()?,
            config,
        });
    }

    wait_for_shutdown().await;
    shutdown(children).await;

    Ok(())
}

struct ManagedChild {
    config: ChildConfig,
    child: Child,
}

#[derive(Deserialize)]
struct ChildConfig {
    name: String,
    command: String,
    #[serde(default)]
    args: Vec<String>,
    #[serde(default)]
    wait_for_postgres: bool,
}

#[derive(Deserialize)]
struct DevelopmentConfig {
    children: Vec<ChildConfig>,
}

impl ChildConfig {
    fn spawn(&self) -> io::Result<Child> {
        Command::new(&self.command)
            .args(&self.args)
            .stdin(Stdio::null())
            .spawn()
    }
}

async fn load_config() -> Result<DevelopmentConfig> {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("src")
        .join("dev")
        .join("development.yml");

    let contents = fs::read_to_string(&path)
        .await
        .context("read development.yml")?;
    let config: DevelopmentConfig =
        serde_saphyr::from_str(&contents).context("parse development.yml")?;
    Ok(config)
}

async fn wait_for_shutdown() {
    let mut sig_int = unix_signal(SignalKind::interrupt()).ok();
    let mut sig_term = unix_signal(SignalKind::terminate()).ok();
    let mut sig_quit = unix_signal(SignalKind::quit()).ok();

    tokio::select! {
        _ = signal::ctrl_c() => {},
        _ = async {
            if let Some(sig) = sig_int.as_mut() {
                sig.recv().await;
            }
        } => {},
        _ = async {
            if let Some(sig) = sig_term.as_mut() {
                sig.recv().await;
            }
        } => {},
        _ = async {
            if let Some(sig) = sig_quit.as_mut() {
                sig.recv().await;
            }
        } => {},
    }
}

async fn shutdown(mut children: Vec<ManagedChild>) {
    println!("⏳ Shutting down development environment (expect docker/postgres)...");

    for managed in &mut children {
        let _ = managed.child.kill().await;
    }

    for mut managed in children {
        let _ = managed.child.wait().await;
        println!(" ⏹ Stopped {}", managed.config.name);
    }
}
