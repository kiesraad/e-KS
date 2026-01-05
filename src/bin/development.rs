use std::{
    fs, io,
    path::PathBuf,
    process::{Child, Command, Stdio},
};

use anyhow::{Context, Result};
use serde::Deserialize;
use tokio::{
    signal,
    signal::unix::{SignalKind, signal as unix_signal},
};

#[path = "../dev/utils.rs"]
mod utils;

use utils::{run_status, stop_running_containers, wait_for_postgres};

struct ManagedChild {
    name: String,
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

#[tokio::main]
async fn main() -> Result<()> {
    println!("Starting development environment...");

    stop_running_containers().context("stop running containers")?;
    run_status(Command::new("docker").args(["compose", "up", "-d"]))
        .context("start docker compose containers")?;

    let config = load_config().context("load development config")?;
    let mut children = Vec::new();
    let mut waited_for_postgres = false;

    for child_config in config.children {
        if child_config.wait_for_postgres && !waited_for_postgres {
            wait_for_postgres(
                "Waiting for PostgreSQL...",
                Some("PostgreSQL is ready, starting cargo watch..."),
                false,
            )
            .await
            .context("wait for postgres")?;
            waited_for_postgres = true;
        }

        let args: Vec<&str> = child_config.args.iter().map(String::as_str).collect();
        children.push(ManagedChild {
            name: child_config.name,
            child: spawn_child(&child_config.command, &args)?,
        });
    }

    wait_for_shutdown().await;
    shutdown(children).await;

    Ok(())
}

fn spawn_child(program: &str, args: &[&str]) -> io::Result<Child> {
    Command::new(program)
        .args(args)
        .stdin(Stdio::null())
        .spawn()
}

fn load_config() -> Result<DevelopmentConfig> {
    let path = config_path();
    let contents = fs::read_to_string(&path).with_context(|| format!("read {}", path.display()))?;
    let config: DevelopmentConfig =
        serde_saphyr::from_str(&contents).context("parse development.yml")?;
    Ok(config)
}

fn config_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("src")
        .join("dev")
        .join("development.yml")
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
    println!("Shutting down development environment...");
    let _ = Command::new("docker").args(["compose", "stop"]).status();

    for managed in &mut children {
        let _ = managed.child.kill();
    }

    for mut managed in children {
        let _ = managed.child.wait();
        println!("Stopped {}.", managed.name);
    }
}
