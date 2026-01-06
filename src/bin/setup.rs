use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use serde::Deserialize;
use tokio::{fs, process::Command};

#[path = "../dev/utils.rs"]
mod utils;

use utils::{platform_string, pts, run, stop_running_containers, temp_dir, wait_for_postgres};

#[tokio::main]
async fn main() -> Result<()> {
    let platform = platform_string().await.context("detect platform")?;
    let config = load_config().await.context("load setup config")?;
    let tools_dir = Path::new("tools");

    fs::create_dir_all(tools_dir)
        .await
        .context("create tools directory")?;

    for tool in config.tools {
        tool.verify_installed(&platform, tools_dir).await?;
    }

    println!("🚀 Setting up Docker containers...");
    stop_running_containers().await?;

    config.commands.docker_compose_rm.run().await?;
    config.commands.docker_compose_up.run().await?;

    println!("📦 Bundling frontend assets with esbuild...");
    config.commands.esbuild_bundle.run().await?;

    println!("📚 Installing cargo-watch (if it is not yet installed)...");
    config.commands.install_cargo_watch.run().await?;

    wait_for_postgres().await?;

    println!("🚚 Running sqlx migrations and loading fixtures...");
    config.commands.load_fixtures.run().await?;

    println!("✅ Setup complete!");
    println!("You can now run 'cargo run --bin development' to start the development environment.");

    Ok(())
}

#[derive(Deserialize)]
struct ToolConfig {
    name: String,
    version: String,
    base_url: String,
}

#[derive(Deserialize)]
struct CommandConfig {
    command: String,
    args: Vec<String>,
}

#[derive(Deserialize)]
struct CommandsConfig {
    docker_compose_rm: CommandConfig,
    docker_compose_up: CommandConfig,
    install_cargo_watch: CommandConfig,
    esbuild_bundle: CommandConfig,
    load_fixtures: CommandConfig,
}

#[derive(Deserialize)]
struct SetupConfig {
    tools: Vec<ToolConfig>,
    commands: CommandsConfig,
}

async fn load_config() -> Result<SetupConfig> {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src/dev/setup.yml");
    let contents = fs::read_to_string(&path).await.context("read setup.yml")?;
    let config: SetupConfig = serde_saphyr::from_str(&contents).context("parse setup.yml")?;

    Ok(config)
}

impl CommandConfig {
    async fn run(&self) -> Result<()> {
        let status = Command::new(&self.command)
            .args(&self.args)
            .status()
            .await?;

        if !status.success() {
            anyhow::bail!("command failed: {:?}", self.command);
        }

        Ok(())
    }
}

impl ToolConfig {
    async fn verify_installed(&self, platform: &str, tools_dir: &Path) -> Result<()> {
        let target = tools_dir.join(&self.name);

        if fs::try_exists(&target).await? {
            println!("✅ {} already installed", self.name);
        } else {
            println!("📦 Installing {} for platform: {platform}", self.name);
            self.install(platform, &target)
                .await
                .context(format!("install {}", self.name))?;
            println!("✅ {} installed", self.name);
        }

        Ok(())
    }

    async fn install(&self, platform: &str, target: &Path) -> Result<()> {
        match self.name.as_str() {
            "esbuild" => install_esbuild(platform, target, self).await,
            "biome" => install_biome(platform, target, self).await,
            "bag-service" => install_bag_service(platform, target, self).await,
            _ => anyhow::bail!("unknown tool: {}", self.name),
        }
    }
}

async fn install_esbuild(platform: &str, target: &Path, tool: &ToolConfig) -> Result<()> {
    let temp_dir = temp_dir().await.context("create temp dir")?;
    let temp_esbuild = temp_dir.join(format!("esbuild-{}.tgz", tool.version));
    let platform_suffix = match platform {
        "Darwin arm64" => "darwin-arm64",
        "Darwin x86_64" => "darwin-x64",
        "Linux arm64" | "Linux aarch64" => "linux-arm64",
        "Linux x86_64" => "linux-x64",
        _ => anyhow::bail!("unsupported platform: {platform}"),
    };
    let url = format!(
        "{}/{}/-/{platform_suffix}-{}.tgz",
        tool.base_url, platform_suffix, tool.version
    );

    run("curl", &["-fo", pts(&temp_esbuild)?, &url]).await?;

    println!("📂 Extracting esbuild...");
    run(
        "tar",
        &[
            "-xzf",
            pts(&temp_esbuild)?,
            "-C",
            pts(&temp_dir)?,
            "package/bin/esbuild",
        ],
    )
    .await?;

    let from = temp_dir.join("package/bin/esbuild");
    fs::copy(&from, target)
        .await
        .context("move esbuild into tools directory")?;
    fs::remove_dir_all(&temp_dir)
        .await
        .context("remove temporary directory")?;

    Ok(())
}

async fn install_biome(platform: &str, target: &Path, tool: &ToolConfig) -> Result<()> {
    let platform_suffix = match platform {
        "Darwin arm64" => "biome-darwin-arm64",
        "Darwin x86_64" => "biome-darwin-x64",
        "Linux arm64" | "Linux aarch64" => "biome-linux-arm64-musl",
        "Linux x86_64" => "biome-linux-x64-musl",
        _ => anyhow::bail!("unsupported platform: {platform}"),
    };
    let url = format!("{}{}/{}", tool.base_url, tool.version, platform_suffix);

    run("curl", &["-Lfo", pts(target)?, &url])
        .await
        .context("download biome")?;
    run("chmod", &["+x", pts(target)?])
        .await
        .context("mark biome as executable")?;

    Ok(())
}

async fn install_bag_service(platform: &str, target: &Path, tool: &ToolConfig) -> Result<()> {
    let platform_suffix = match platform {
        "Darwin arm64" => "bag-service-macos-arm64",
        "Darwin x86_64" => "bag-service-macos-x64",
        "Linux arm64" | "Linux aarch64" => "bag-service-linux-arm64",
        "Linux x86_64" => "bag-service-linux-x64",
        _ => anyhow::bail!("unsupported platform: {platform}"),
    };
    let url = format!("{}/{}/{}", tool.base_url, tool.version, platform_suffix);

    run("curl", &["-Lfo", pts(target)?, &url])
        .await
        .context("download bag-service")?;
    run("chmod", &["+x", pts(target)?])
        .await
        .context("mark bag-service as executable")?;

    Ok(())
}
