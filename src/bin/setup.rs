use std::{fs, path::Path, process::Command};

use anyhow::{Context, Result};

#[path = "../dev/utils.rs"]
mod utils;

use utils::{platform_string, run_status, stop_running_containers, temp_dir, wait_for_postgres};

const ESBUILD_VERSION: &str = "0.27.1";
const BIOME_VERSION: &str = "2.3.10";
const BAG_SERVICE_VERSION: &str = "0.3.1";
const ESBUILD_BASE_URL: &str = "https://registry.npmjs.org/@esbuild";
const BIOME_BASE_URL: &str = "https://github.com/biomejs/biome/releases/download/@biomejs/biome@";
const BAG_SERVICE_BASE_URL: &str =
    "https://github.com/tweedegolf/bag-address-lookup/releases/download";

#[tokio::main]
async fn main() -> Result<()> {
    let platform = platform_string().context("detect platform")?;
    let temp_dir = temp_dir().context("create temp dir")?;

    let tools_dir = Path::new("tools");
    fs::create_dir_all(tools_dir)
        .with_context(|| format!("create tools directory at {}", tools_dir.display()))?;

    let esbuild_target = tools_dir.join("esbuild");
    let biome_target = tools_dir.join("biome");
    let bag_service_target = tools_dir.join("bag-service");

    if !esbuild_target.exists() {
        println!("📦 Installing esbuild for platform: {platform}");
        install_esbuild(&platform, &temp_dir, &esbuild_target)?;
    }
    println!("✅ esbuild installed");

    if !biome_target.exists() {
        println!("📦 Installing Biome for platform: {platform}");
        install_biome(&platform, &biome_target)?;
    }
    println!("✅ Biome installed");

    if !bag_service_target.exists() {
        println!("📦 Installing bag-service for platform: {platform}");
        install_bag_service(&platform, &bag_service_target)?;
    }
    println!("✅ bag-address-lookup installed");

    println!("🚀 Setting up Docker containers...");
    stop_running_containers().context("stop running containers")?;
    run_status(Command::new("docker").args(["compose", "rm", "-f"]))
        .context("remove docker compose containers")?;
    run_status(Command::new("docker").args(["compose", "up", "-d"]))
        .context("start docker compose containers")?;

    run_status(Command::new("./tools/esbuild").args([
        "--bundle",
        "frontend/index.ts",
        "--outdir=frontend/static",
        "--minify",
        "--sourcemap",
        "--define:IS_PRODUCTION=true",
        "--loader:.woff2=file",
        "--loader:.svg=file",
        "--public-path=/static/",
    ]))
    .context("run esbuild")?;

    wait_for_postgres("⏳ Waiting for PostgreSQL...", None, true)
        .await
        .context("wait for postgres")?;

    println!("🚚 Running sqlx migrations and loading fixtures...");
    run_status(Command::new("cargo").args(["run", "--features", "fixtures", "--bin", "fixtures"]))
        .context("load fixtures")?;

    println!("✅ Setup complete!");
    println!("You can now run 'cargo run --bin development' to start the development environment.");

    Ok(())
}

fn install_esbuild(platform: &str, temp_dir: &Path, target: &Path) -> Result<()> {
    let temp_esbuild = temp_dir.join(format!("esbuild-{ESBUILD_VERSION}.tgz"));
    let platform_suffix = match platform {
        "Darwin arm64" => "darwin-arm64",
        "Darwin x86_64" => "darwin-x64",
        "Linux arm64" | "Linux aarch64" => "linux-arm64",
        "Linux x86_64" => "linux-x64",
        _ => anyhow::bail!("unsupported platform: {platform}"),
    };
    let url =
        format!("{ESBUILD_BASE_URL}/{platform_suffix}/-/{platform_suffix}-{ESBUILD_VERSION}.tgz");

    run_status(Command::new("curl").args(["-fo", temp_esbuild.to_str().unwrap(), &url]))
        .with_context(|| format!("download esbuild from {url}"))?;

    println!("📂 Extracting esbuild...");
    run_status(Command::new("tar").args([
        "-xzf",
        temp_esbuild.to_str().unwrap(),
        "-C",
        temp_dir.to_str().unwrap(),
        "package/bin/esbuild",
    ]))
    .context("extract esbuild archive")?;

    let from = temp_dir.join("package/bin/esbuild");

    fs::copy(from, target).context("move esbuild into tools directory")?;
    fs::remove_dir_all(temp_dir).context("remove temporary directory")?;

    Ok(())
}

fn install_biome(platform: &str, target: &Path) -> Result<()> {
    let platform_suffix = match platform {
        "Darwin arm64" => "biome-darwin-arm64",
        "Darwin x86_64" => "biome-darwin-x64",
        "Linux arm64" | "Linux aarch64" => "biome-linux-arm64-musl",
        "Linux x86_64" => "biome-linux-x64-musl",
        _ => anyhow::bail!("unsupported platform: {platform}"),
    };
    let url = format!("{BIOME_BASE_URL}{BIOME_VERSION}/{platform_suffix}");

    run_status(Command::new("curl").args(["-Lfo", target.to_str().unwrap(), &url]))
        .with_context(|| format!("download biome from {url}"))?;
    run_status(Command::new("chmod").args(["+x", target.to_str().unwrap()]))
        .context("mark biome as executable")?;
    Ok(())
}

fn install_bag_service(platform: &str, target: &Path) -> Result<()> {
    let platform_suffix = match platform {
        "Darwin arm64" => "bag-service-macos-arm64",
        "Darwin x86_64" => "bag-service-macos-x64",
        "Linux arm64" | "Linux aarch64" => "bag-service-linux-arm64",
        "Linux x86_64" => "bag-service-linux-x64",
        _ => anyhow::bail!("unsupported platform: {platform}"),
    };
    let url = format!("{BAG_SERVICE_BASE_URL}/{BAG_SERVICE_VERSION}/{platform_suffix}");

    run_status(Command::new("curl").args(["-Lfo", target.to_str().unwrap(), &url]))
        .with_context(|| format!("download bag-service from {url}"))?;
    run_status(Command::new("chmod").args(["+x", target.to_str().unwrap()]))
        .context("mark bag-service as executable")?;
    Ok(())
}
