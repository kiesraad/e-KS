use std::path::{Path, PathBuf};

use anyhow::Result;
use tokio::{
    process::Command,
    time::{Duration, sleep, timeout},
};

pub async fn run(command: &str, args: &[&str]) -> Result<()> {
    println!("$> {command} {}", args.join(" "));
    let status = Command::new(command).args(args).status().await?;

    if !status.success() {
        anyhow::bail!("command failed: {:?}", command);
    }

    Ok(())
}

pub async fn stop_running_containers() -> Result<()> {
    let output = Command::new("docker").args(["ps", "-q"]).output().await?;
    if output.status.success() {
        let ids = String::from_utf8_lossy(&output.stdout);
        let ids: Vec<&str> = ids.split_whitespace().collect();
        if !ids.is_empty() {
            let mut args = Vec::with_capacity(ids.len() + 1);
            args.push("kill");
            args.extend(ids.iter().cloned());
            run("docker", &args).await?;
        }
    }
    Ok(())
}

pub async fn wait_for_postgres() -> Result<()> {
    for _ in 0..20 {
        let attempt = timeout(
            Duration::from_secs(1),
            Command::new("docker")
                .args(["compose", "exec", "-T", "psql", "pg_isready", "-U", "eks"])
                .status(),
        )
        .await;

        if matches!(attempt, Ok(Ok(status)) if status.success()) {
            println!("✅ PostgreSQL is up!");

            // small delay to ensure connecting from outside docker works
            sleep(Duration::from_millis(500)).await;

            return Ok(());
        }

        println!("⏳ Waiting for PostgreSQL...");
        sleep(Duration::from_secs(1)).await;
    }

    anyhow::bail!("PostgreSQL did not start in time");
}

pub fn pts(path: &Path) -> Result<&str> {
    path.to_str()
        .ok_or_else(|| anyhow::anyhow!("convert {path:?} to str"))
}

pub async fn platform_string() -> Result<String> {
    let output = Command::new("uname").args(["-ms"]).output().await?;

    if !output.status.success() {
        anyhow::bail!("uname -ms failed");
    }

    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

pub async fn temp_dir() -> Result<PathBuf> {
    let output = Command::new("mktemp").arg("-d").output().await?;

    if !output.status.success() {
        anyhow::bail!("mktemp -d failed");
    }

    Ok(PathBuf::from(
        String::from_utf8_lossy(&output.stdout).trim(),
    ))
}
