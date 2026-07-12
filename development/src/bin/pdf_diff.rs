//! Render every PDF model for every JSON example input
//! (`src/models/example-inputs/`) and compare the output against a baseline.
//!
//! The models are Rust code compiled into the `eks` crate, so unlike the old
//! Typst-based tool this cannot render the `main` branch's models directly.
//! Instead it diffs against a saved baseline:
//!
//! 1. on `main`: `cargo run --bin pdf_diff -- --save-baseline`
//!    (renders into `tmp/main-pdfs/`)
//! 2. on your branch: `cargo run --bin pdf_diff`
//!    (renders into `tmp/current-pdfs/`, writes visual diffs of changed PDFs
//!    to `tmp/diffs/` and a summary to `tmp/results.md`)
//!
//! Requires `diff-pdf` (`apt-get install diff-pdf-wx` / `brew install diff-pdf`).

use std::{
    collections::BTreeSet,
    env,
    fmt::Write as _,
    fs,
    path::{Path, PathBuf},
    process::Command,
};

use anyhow::{Context, Result};
use tracing::info;
use tracing_subscriber::{Layer, layer::SubscriberExt, util::SubscriberInitExt};

const EXAMPLE_INPUTS_DIR: &str = "src/models/example-inputs";
const DIFF_PDF_BINARY: &str = "diff-pdf";
const TMP_DIR_NAME: &str = "tmp";
const CURRENT_PDFS_DIR_NAME: &str = "current-pdfs";
const MAIN_PDFS_DIR_NAME: &str = "main-pdfs";
const DIFFS_DIR_NAME: &str = "diffs";
const RESULTS_FILE_NAME: &str = "results.md";

type DiffSummaryRow = (String, String, &'static str);

/// Return whether a binary name resolves to an executable file somewhere on `PATH`.
fn binary_in_path(name: &str) -> bool {
    if let Ok(paths) = env::var("PATH") {
        for dir in env::split_paths(&paths) {
            if dir.join(name).is_file() {
                return true;
            }
        }
    }
    false
}

/// Ensure the `diff-pdf` executable is available before starting any work.
fn ensure_diff_pdf_is_installed() -> Result<()> {
    if !binary_in_path(DIFF_PDF_BINARY) {
        // `sudo apt-get install -y diff-pdf-wx` or `brew install diff-pdf`
        anyhow::bail!(
            "`diff-pdf` is not installed or not found in PATH. Please install it to use this tool."
        );
    }
    Ok(())
}

/// The template name for an example input file, e.g.
/// `model-h3-1-example-2.json` → `model-h3-1`.
fn template_name(input: &Path) -> Option<String> {
    let stem = input.file_stem()?.to_str()?;
    let (template, _) = stem.split_once("-example-")?;
    Some(template.to_string())
}

/// Collect the example input files, sorted by name.
fn example_inputs(project_dir: &Path) -> Result<Vec<PathBuf>> {
    let dir = project_dir.join(EXAMPLE_INPUTS_DIR);
    let mut inputs = Vec::new();
    for entry in fs::read_dir(&dir).with_context(|| format!("Failed to read {}", dir.display()))? {
        let path = entry?.path();
        if path.extension().and_then(|ext| ext.to_str()) == Some("json") {
            inputs.push(path);
        }
    }
    inputs.sort_unstable();
    Ok(inputs)
}

/// Render every example input into `output_root` and return the written PDF
/// file names.
fn render_all(project_dir: &Path, output_root: &Path) -> Result<Vec<String>> {
    fs::create_dir_all(output_root)
        .with_context(|| format!("Failed to create {}", output_root.display()))?;

    let mut rendered = Vec::new();
    for input_path in example_inputs(project_dir)? {
        let Some(template) = template_name(&input_path) else {
            continue;
        };
        let json = fs::read_to_string(&input_path)
            .with_context(|| format!("Failed to read {}", input_path.display()))?;
        let input: serde_json::Value = serde_json::from_str(&json)
            .with_context(|| format!("Failed to parse {}", input_path.display()))?;

        let bytes = eks::models::render_example(&template, input)
            .map_err(|err| anyhow::anyhow!("{err}"))
            .with_context(|| format!("Failed to render {}", input_path.display()))?;

        let pdf_name = format!(
            "{}.pdf",
            input_path
                .file_stem()
                .and_then(|stem| stem.to_str())
                .context("input file name")?
        );
        let output = output_root.join(&pdf_name);
        fs::write(&output, &bytes)
            .with_context(|| format!("Failed to write {}", output.display()))?;
        info!(
            template,
            output = %output.display(),
            pdf_size_bytes = bytes.len(),
            "Rendered PDF"
        );
        rendered.push(pdf_name);
    }
    Ok(rendered)
}

/// Run `diff-pdf` for two rendered PDFs and report whether their contents differ.
fn diff_pdfs(current_pdf: &Path, main_pdf: &Path, diff_pdf: &Path) -> Result<bool> {
    let status = Command::new(DIFF_PDF_BINARY)
        .arg(format!("--output-diff={}", diff_pdf.display()))
        .arg("--skip-identical")
        .arg(main_pdf)
        .arg(current_pdf)
        .status()
        .context("Failed to run diff-pdf")?;

    match status.code() {
        Some(0) => Ok(false),
        Some(1) => Ok(true),
        Some(code) => anyhow::bail!("diff-pdf failed with exit code {code}"),
        None => anyhow::bail!("diff-pdf terminated by signal"),
    }
}

fn status_indicator(status: &str) -> &'static str {
    match status {
        "added" => "🟢",
        "deleted" => "🔴",
        "changed" => "🟠",
        "identical" => "🔵",
        _ => "⚪",
    }
}

/// Build the Markdown summary table written to `tmp/results.md`.
fn build_report(results: &[DiffSummaryRow]) -> Result<String> {
    let mut report = String::new();
    writeln!(report, "| Template | Input | Status |")?;
    writeln!(report, "| --- | --- | --- |")?;
    for (template, input_name, status) in results {
        writeln!(
            report,
            "| {template} | {input_name} | {} {status} |",
            status_indicator(status),
        )?;
    }
    Ok(report)
}

/// Compare the rendered PDFs against the baseline and summarize per input.
fn compare(project_dir: &Path, rendered: Vec<String>) -> Result<Vec<DiffSummaryRow>> {
    let tmp_dir = project_dir.join(TMP_DIR_NAME);
    let current_root = tmp_dir.join(CURRENT_PDFS_DIR_NAME);
    let main_root = tmp_dir.join(MAIN_PDFS_DIR_NAME);
    let diffs_root = tmp_dir.join(DIFFS_DIR_NAME);
    fs::create_dir_all(&diffs_root).context("Failed to create tmp/diffs")?;

    let baseline: BTreeSet<String> = fs::read_dir(&main_root)?
        .filter_map(|entry| entry.ok())
        .filter_map(|entry| entry.file_name().into_string().ok())
        .filter(|name| name.ends_with(".pdf"))
        .collect();

    let mut results = Vec::new();
    for pdf_name in &rendered {
        let template = pdf_name.split("-example-").next().unwrap_or(pdf_name);
        let row = if baseline.contains(pdf_name) {
            let diff_pdf = diffs_root.join(pdf_name);
            let changed = diff_pdfs(
                &current_root.join(pdf_name),
                &main_root.join(pdf_name),
                &diff_pdf,
            )?;
            if !changed {
                let _ = fs::remove_file(&diff_pdf);
            }
            if changed { "changed" } else { "identical" }
        } else {
            "added"
        };
        results.push((template.to_string(), pdf_name.clone(), row));
    }
    for pdf_name in baseline {
        if !rendered.contains(&pdf_name) {
            let template = pdf_name
                .split("-example-")
                .next()
                .unwrap_or(&pdf_name)
                .to_string();
            results.push((template, pdf_name, "deleted"));
        }
    }
    results.sort_unstable();
    Ok(results)
}

/// Initialize tracing for this binary and limit output to this crate's logs.
fn init_tracing() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::fmt::layer().with_filter(tracing_subscriber::filter::filter_fn(
                |metadata| metadata.target().starts_with(env!("CARGO_CRATE_NAME")),
            )),
        )
        .init();
}

fn run() -> Result<()> {
    let project_dir = env::current_dir().context("Failed to get current directory")?;
    let tmp_dir = project_dir.join(TMP_DIR_NAME);
    let save_baseline = env::args().any(|arg| arg == "--save-baseline");

    if save_baseline {
        let main_root = tmp_dir.join(MAIN_PDFS_DIR_NAME);
        let rendered = render_all(&project_dir, &main_root)?;
        info!(
            "Saved {} baseline PDFs to {}",
            rendered.len(),
            main_root.display()
        );
        return Ok(());
    }

    ensure_diff_pdf_is_installed()?;

    let current_root = tmp_dir.join(CURRENT_PDFS_DIR_NAME);
    let rendered = render_all(&project_dir, &current_root)?;
    info!(
        "Rendered {} PDFs to {}",
        rendered.len(),
        current_root.display()
    );

    if !tmp_dir.join(MAIN_PDFS_DIR_NAME).is_dir() {
        info!(
            "No baseline found in tmp/{MAIN_PDFS_DIR_NAME}; run `cargo run --bin pdf_diff -- --save-baseline` on the base branch first"
        );
        return Ok(());
    }

    let results = compare(&project_dir, rendered)?;
    for (_, input_name, status) in &results {
        info!("  {} {input_name}: {status}", status_indicator(status));
    }

    let report = build_report(&results)?;
    let results_path = tmp_dir.join(RESULTS_FILE_NAME);
    fs::write(&results_path, &report)
        .with_context(|| format!("Failed to write {}", results_path.display()))?;
    info!("Wrote results to {}", results_path.display());

    Ok(())
}

/// Render the current PDF models for all example inputs and diff them against
/// the saved baseline.
fn main() -> Result<()> {
    init_tracing();
    run()
}
