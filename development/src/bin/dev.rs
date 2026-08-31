use std::{io, process::Stdio};

use anyhow::{Context, Result};
use serde::Deserialize;
use tokio::{
    process::{Child, Command},
    signal,
    signal::unix::{SignalKind, signal as unix_signal},
};

use eks_development::{dotenv_variables, run, stop_running_containers, wait_for_postgres};

#[tokio::main]
async fn main() -> Result<()> {
    println!("⏳ Starting development environment...");

    stop_running_containers().await?;
    run("docker", &["compose", "up", "-d"]).await?;

    let config = load_config().await?;

    let wild_env = wild_linker_env();
    if !wild_env.is_empty() {
        println!("🔗 Using the wild linker for faster Rust builds");
    }

    let dotenv = dotenv_variables()?;
    if !dotenv.is_empty() {
        let names: Vec<&str> = dotenv.iter().map(|(name, _)| name.as_str()).collect();
        println!("📄 From .env/.env.local: {}", names.join(", "));
    }

    // Handed to each child rather than set on this process: `set_var` is
    // unsafe and the workspace forbids unsafe code.
    let child_env: Vec<(String, String)> = wild_env
        .into_iter()
        .map(|(name, value)| (name.to_string(), value.to_string()))
        .chain(dotenv)
        .collect();

    let mut children = Vec::new();
    let mut waited_for_postgres = false;

    for config in config.children {
        if config.wait_for_postgres && !waited_for_postgres {
            wait_for_postgres().await?;
            waited_for_postgres = true;
        }

        println!("🚀 Starting {}", config.name);
        children.push(ManagedChild {
            child: config.spawn(&child_env)?,
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
    fn spawn(&self, env: &[(String, String)]) -> io::Result<Child> {
        Command::new(&self.command)
            .args(&self.args)
            .envs(env.iter().map(|(name, value)| (name, value)))
            .stdin(Stdio::null())
            .spawn()
    }
}

/// Environment overrides that route Rust builds through the wild linker
/// <https://github.com/davidlattimore/wild>, which links substantially faster
/// than the default GNU ld and shortens the edit-compile-run loop. Wild is
/// driven through clang via `--ld-path`, so both must be on PATH; when either
/// is missing we return no overrides and builds use the default linker. Kept
/// dev-only (here, not in .cargo/config.toml) so release builds via bin/build
/// stay reproducible and need neither tool.
fn wild_linker_env() -> Vec<(&'static str, &'static str)> {
    if is_on_path("wild") && is_on_path("clang") {
        vec![
            ("CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_LINKER", "clang"),
            (
                "CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_RUSTFLAGS",
                "-C link-arg=--ld-path=wild",
            ),
        ]
    } else {
        Vec::new()
    }
}

fn is_on_path(bin: &str) -> bool {
    let Some(path) = std::env::var_os("PATH") else {
        return false;
    };
    std::env::split_paths(&path).any(|dir| dir.join(bin).is_file())
}

async fn load_config() -> Result<DevelopmentConfig> {
    let contents = include_str!("../../development.yml");
    let config: DevelopmentConfig =
        serde_saphyr::from_str(contents).context("parse development.yml")?;
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
