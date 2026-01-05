use std::{
    path::PathBuf,
    process::{Command, Stdio},
    time::Duration,
};

use anyhow::Result;
use tokio::time::sleep;

pub fn run_status(cmd: &mut Command) -> Result<()> {
    let status = cmd.status()?;

    if !status.success() {
        anyhow::bail!("command failed: {:?}", cmd);
    }

    Ok(())
}

pub fn stop_running_containers() -> Result<()> {
    let output = Command::new("docker").args(["ps", "-q"]).output()?;
    if output.status.success() {
        let ids = String::from_utf8_lossy(&output.stdout);
        let ids: Vec<&str> = ids.split_whitespace().collect();
        if !ids.is_empty() {
            let mut cmd = Command::new("docker");
            cmd.arg("kill");
            cmd.args(ids);
            run_status(&mut cmd)?;
        }
    }
    Ok(())
}

pub async fn wait_for_postgres(
    wait_message: &str,
    ready_message: Option<&str>,
    repeat_wait_message: bool,
) -> Result<()> {
    if !repeat_wait_message {
        println!("{wait_message}");
    }

    loop {
        let status = Command::new("pg_isready")
            .args(["-h", "127.0.0.1", "-q"])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()?;
        if status.success() {
            if let Some(message) = ready_message {
                println!("{message}");
            }
            return Ok(());
        }
        if repeat_wait_message {
            println!("{wait_message}");
        }
        sleep(Duration::from_secs(1)).await;
    }
}

#[allow(unused)]
pub fn platform_string() -> Result<String> {
    let output = Command::new("uname").args(["-ms"]).output()?;

    if !output.status.success() {
        anyhow::bail!("uname -ms failed");
    }

    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

#[allow(unused)]
pub fn temp_dir() -> Result<PathBuf> {
    let output = Command::new("mktemp").arg("-d").output()?;

    if !output.status.success() {
        anyhow::bail!("mktemp -d failed");
    }

    Ok(PathBuf::from(
        String::from_utf8_lossy(&output.stdout).trim(),
    ))
}
