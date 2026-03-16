use anyhow::{Context, Result};
use std::{
    collections::HashSet,
    env,
    fmt::Write as _,
    path::{Path, PathBuf},
    sync::Arc,
};
use tokio::process::Command;
use tracing::{info, trace};
use tracing_subscriber::{Layer, layer::SubscriberExt, util::SubscriberInitExt};
use typst_webservice::PdfContext;

const MODEL_FILE_PREFIX: &str = "model-";
const EXAMPLE_INPUTS_DIR: &str = "example-inputs";
const DIFF_PDF_BINARY: &str = "diff-pdf";
const TMP_DIR_NAME: &str = "tmp";
const MODELS_DIR_NAME: &str = "models";
const CURRENT_MODELS_DIR_NAME: &str = "current-models";
const MAIN_MODELS_DIR_NAME: &str = "main-models";
const DIFFS_DIR_NAME: &str = "diffs";
const CURRENT_PDFS_DIR_NAME: &str = "current-pdfs";
const MAIN_PDFS_DIR_NAME: &str = "main-pdfs";
const GIT_MAIN_REF: &str = "origin/main";
const RESULTS_FILE_NAME: &str = "results.md";

type DiffSummaryRow = (PathBuf, Option<String>, &'static str, Option<PathBuf>);

struct WorkspacePaths {
    tmp_dir: PathBuf,
    models_dir: PathBuf,
    current_root: PathBuf,
    main_root: PathBuf,
    diffs_root: PathBuf,
    current_pdfs_root: PathBuf,
    main_pdfs_root: PathBuf,
    results_path: PathBuf,
}

impl WorkspacePaths {
    /// Build the derived workspace paths used while generating and comparing PDFs.
    fn new(project_dir: PathBuf) -> Self {
        let tmp_dir = project_dir.join(TMP_DIR_NAME);
        Self {
            models_dir: project_dir.join(MODELS_DIR_NAME),
            current_root: tmp_dir.join(CURRENT_MODELS_DIR_NAME),
            main_root: tmp_dir.join(MAIN_MODELS_DIR_NAME),
            diffs_root: tmp_dir.join(DIFFS_DIR_NAME),
            current_pdfs_root: tmp_dir.join(CURRENT_PDFS_DIR_NAME),
            main_pdfs_root: tmp_dir.join(MAIN_PDFS_DIR_NAME),
            results_path: tmp_dir.join(RESULTS_FILE_NAME),
            tmp_dir,
        }
    }
}

struct FileGroups {
    new_files: Vec<PathBuf>,
    deleted_files: Vec<PathBuf>,
    common_files: Vec<PathBuf>,
}

/// Return whether a binary name resolves to an executable file somewhere on `PATH`.
fn binary_in_path(name: &str) -> bool {
    if let Ok(paths) = env::var("PATH") {
        for dir in env::split_paths(&paths) {
            let full = dir.join(name);
            if full.is_file() {
                return true;
            }
        }
    }
    false
}

/// Collect all model Typst files below `root`, returning paths relative to that root.
fn collect_typst_files(root: &Path) -> Result<Vec<PathBuf>> {
    let mut files = Vec::new();
    let mut stack = vec![root.to_path_buf()];

    while let Some(dir) = stack.pop() {
        tracing::debug!("Scanning directory {}", dir.display());

        for entry in std::fs::read_dir(&dir)
            .with_context(|| format!("Failed to read directory {}", dir.display()))?
        {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                stack.push(path);
            } else if path.extension().and_then(|ext| ext.to_str()) == Some("typ")
                && path
                    .file_name()
                    .map(|name| name.to_string_lossy().starts_with(MODEL_FILE_PREFIX))
                    == Some(true)
            {
                files.push(
                    path.strip_prefix(root)
                        .with_context(|| format!("Failed to strip prefix {}", root.display()))?
                        .to_path_buf(),
                );
            }
        }
    }

    Ok(files)
}

/// Render a template with the provided JSON input and write the resulting PDF to `output`.
async fn render_pdf(
    context: Arc<PdfContext>,
    template: String,
    input: serde_json::Value,
    file: &Path,
    output: &Path,
) -> Result<()> {
    trace!(
        template = template,
        file = %file.display(),
        output = %output.display(),
        "Rendering typst file"
    );
    let bytes = tokio::task::spawn_blocking(move || PdfContext::render(context, template, input))
        .await
        .context("Typst render task panicked")?
        .with_context(|| format!("Failed to render PDF for {}", file.display()))?;

    trace!(
        file = %file.display(),
        output = %output.display(),
        pdf_size_bytes = bytes.len(),
        "Rendered PDF"
    );

    if let Some(parent) = output.parent() {
        tokio::fs::create_dir_all(parent)
            .await
            .with_context(|| format!("Failed to create {}", parent.display()))?;
    }

    tokio::fs::write(output, &bytes)
        .await
        .with_context(|| format!("Failed to write {}", output.display()))?;

    Ok(())
}

/// Extract the template file name from a relative model path.
fn template_name(file: &Path) -> Result<String> {
    file.file_name()
        .and_then(|name| name.to_str())
        .map(str::to_owned)
        .with_context(|| format!("Failed to determine template name for {}", file.display()))
}

/// Load the example JSON input associated with a model template from `example-inputs/`.
async fn load_render_inputs(
    root: &Path,
    template: &str,
) -> Result<Vec<(String, serde_json::Value)>> {
    let prefix = template.replace(".typ", "");

    let path = root.join(EXAMPLE_INPUTS_DIR);

    // loop trough `path` and collect all files with extension `.json` and name starting with `prefix`
    let mut inputs = Vec::new();

    for entry in std::fs::read_dir(&path)
        .with_context(|| format!("Failed to read directory {}", path.display()))?
    {
        let entry = entry?;
        let path = entry.path();
        if path.extension().and_then(|ext| ext.to_str()) == Some("json") {
            let file_name = path
                .file_name()
                .and_then(|name| name.to_str())
                .unwrap_or_default()
                .to_string();

            if file_name.starts_with(&prefix) {
                let input = tokio::fs::read_to_string(&path)
                    .await
                    .with_context(|| format!("Failed to read {}", path.display()))?;

                let json = serde_json::from_str(&input)
                    .with_context(|| format!("Failed to parse JSON from {}", path.display()))?;

                inputs.push((file_name, json));
            }
        }
    }

    Ok(inputs)
}

/// Run `diff-pdf` for two rendered PDFs and report whether their contents differ.
async fn diff_pdfs(current_pdf: &Path, main_pdf: &Path, diff_pdf: &Path) -> Result<bool> {
    let status = Command::new(DIFF_PDF_BINARY)
        .arg(format!("--output-diff={}", diff_pdf.display()))
        .arg("--skip-identical")
        .arg(main_pdf)
        .arg(current_pdf)
        .status()
        .await
        .context("Failed to run diff-pdf")?;

    match status.code() {
        Some(0) => Ok(false),
        Some(1) => Ok(true),
        Some(code) => anyhow::bail!("diff-pdf failed with exit code {code}"),
        None => anyhow::bail!("diff-pdf terminated by signal"),
    }
}

/// Run a subprocess and fail with captured stderr when it exits unsuccessfully.
async fn run_command(command: &str, args: &[&str], error_message: &str) -> Result<()> {
    let output = Command::new(command)
        .args(args)
        .output()
        .await
        .with_context(|| format!("Failed to run `{command}`"))?;

    if output.status.success() {
        return Ok(());
    }

    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_owned();
    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_owned();
    let details = if !stderr.is_empty() {
        stderr
    } else if !stdout.is_empty() {
        stdout
    } else {
        format!("command exited with status {}", output.status)
    };

    anyhow::bail!("{error_message}: {details}");
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

/// Remove the temporary working directory when it exists from a previous run.
async fn clean_tmp_dir(tmp_dir: &Path) -> Result<()> {
    if tmp_dir.exists() {
        run_command(
            "rm",
            &["-rf", tmp_dir.to_string_lossy().as_ref()],
            "Failed to clean tmp directory",
        )
        .await?;
    }

    Ok(())
}

/// Prepare the temporary workspace by separating current and `main` model trees.
async fn prepare_workspace(paths: &WorkspacePaths) -> Result<()> {
    clean_tmp_dir(&paths.tmp_dir).await?;

    tokio::fs::create_dir_all(&paths.tmp_dir)
        .await
        .context("Failed to create tmp directory")?;

    tokio::fs::create_dir_all(&paths.main_root)
        .await
        .context("Failed to create tmp directory")?;

    tokio::fs::rename(&paths.models_dir, &paths.current_root)
        .await
        .context("Failed to move models/ to tmp/current-models/")?;

    run_command(
        "git",
        &["checkout", GIT_MAIN_REF, "--", MODELS_DIR_NAME],
        "Failed to checkout models/ from main branch",
    )
    .await?;

    tokio::fs::rename(&paths.models_dir, &paths.main_root)
        .await
        .context("Failed to move models/ to tmp/main-models/")?;

    run_command(
        "cp",
        &["-r", paths.current_root.to_string_lossy().as_ref(), "."],
        "Failed to restore models/ from tmp/current-models/",
    )
    .await?;

    tokio::fs::rename(Path::new(CURRENT_MODELS_DIR_NAME), &paths.models_dir)
        .await
        .context("Failed to rename tmp/current-models/ to models/")?;

    Ok(())
}

/// Partition model files into added, deleted, and shared groups.
fn group_model_files(current_root: &Path, main_root: &Path) -> Result<FileGroups> {
    let current_files = collect_typst_files(current_root)?;
    tracing::info!(
        "Found {} typst files in current branch",
        current_files.len()
    );

    let main_files = collect_typst_files(main_root)?;
    tracing::info!("Found {} typst files in main branch", main_files.len());

    let current_set: HashSet<PathBuf> = current_files.into_iter().collect();
    let main_set: HashSet<PathBuf> = main_files.into_iter().collect();

    let mut new_files = Vec::new();
    let mut deleted_files = Vec::new();
    let mut common_files = Vec::new();

    for file in &current_set {
        if main_set.contains(file) {
            common_files.push(file.clone());
        } else {
            new_files.push(file.clone());
        }
    }

    for file in &main_set {
        if !current_set.contains(file) {
            deleted_files.push(file.clone());
        }
    }

    new_files.sort_unstable();
    deleted_files.sort_unstable();
    common_files.sort_unstable();

    Ok(FileGroups {
        new_files,
        deleted_files,
        common_files,
    })
}

/// Log one named group of files using the provided status icon.
fn log_file_group(label: &str, icon: &str, files: &[PathBuf]) {
    if files.is_empty() {
        return;
    }

    info!("{label}:");
    for file in files {
        info!("  {icon} {}", file.display());
    }
}

/// Log all grouped model file changes.
fn log_file_groups(groups: &FileGroups) {
    log_file_group("New files", "🟢", &groups.new_files);
    log_file_group("Deleted files", "🔴", &groups.deleted_files);
    log_file_group("Common files", "🔵", &groups.common_files);
}

/// Load Typst contexts for the current branch and the checked-out `main` branch.
async fn load_pdf_contexts(paths: &WorkspacePaths) -> Result<(Arc<PdfContext>, Arc<PdfContext>)> {
    tokio::fs::create_dir_all(&paths.diffs_root)
        .await
        .context("Failed to create tmp/diffs")?;

    let current_context = Arc::new(
        PdfContext::from_directory(&paths.current_root)
            .with_context(|| format!("Failed to load {}", paths.current_root.display()))?,
    );
    let main_context = Arc::new(
        PdfContext::from_directory(&paths.main_root)
            .with_context(|| format!("Failed to load {}", paths.main_root.display()))?,
    );
    info!("Typst contexts loaded");

    Ok((current_context, main_context))
}

/// Render PDFs for newly added templates and return their summary rows.
async fn summarize_new_files(
    paths: &WorkspacePaths,
    current_context: &Arc<PdfContext>,
    new_files: Vec<PathBuf>,
) -> Result<Vec<DiffSummaryRow>> {
    let mut results = Vec::new();

    for rel in new_files {
        let template = template_name(&rel)?;
        let pdf_rel = rel.with_extension("pdf");
        let added_pdf = paths.diffs_root.join(&pdf_rel);

        let inputs = load_render_inputs(&paths.current_root, &template).await?;

        for (input_name, input) in inputs {
            render_pdf(
                Arc::clone(current_context),
                template.clone(),
                input,
                &rel,
                &added_pdf,
            )
            .await?;

            results.push((
                rel.clone(),
                Some(input_name),
                "added",
                Some(Path::new(DIFFS_DIR_NAME).join(pdf_rel.clone())),
            ));
        }
    }

    Ok(results)
}

/// Convert deleted templates into summary rows without generating PDF artifacts.
fn summarize_deleted_files(deleted_files: Vec<PathBuf>) -> Vec<DiffSummaryRow> {
    deleted_files
        .into_iter()
        .map(|rel| (rel, None, "deleted", None))
        .collect()
}

/// Render and compare PDFs for templates that exist in both branches.
async fn summarize_common_files(
    paths: &WorkspacePaths,
    current_context: &Arc<PdfContext>,
    main_context: &Arc<PdfContext>,
    common_files: Vec<PathBuf>,
) -> Result<Vec<DiffSummaryRow>> {
    let mut results = Vec::new();

    for rel in common_files {
        let template = template_name(&rel)?;
        let pdf_rel = rel.with_extension("pdf");
        let current_pdf = paths.current_pdfs_root.join(&pdf_rel);
        let main_pdf = paths.main_pdfs_root.join(&pdf_rel);
        let diff_pdf = paths.diffs_root.join(&pdf_rel);

        let inputs = load_render_inputs(&paths.current_root, &template).await?;

        for (input_name, input) in inputs {
            render_pdf(
                Arc::clone(current_context),
                template.clone(),
                input,
                &rel,
                &current_pdf,
            )
            .await?;

            let inputs = load_render_inputs(&paths.main_root, &template).await?;
            let input_tuple = inputs.into_iter().find(|(name, _)| name == &input_name);

            if let Some((_, input)) = input_tuple {
                render_pdf(
                    Arc::clone(main_context),
                    template.clone(),
                    input,
                    &rel,
                    &main_pdf,
                )
                .await?;

                let changed = diff_pdfs(&current_pdf, &main_pdf, &diff_pdf).await?;
                if !changed {
                    let _ = tokio::fs::remove_file(&diff_pdf).await;
                }
                let status = if changed { "changed" } else { "identical" };
                let diff_link = changed.then(|| Path::new(DIFFS_DIR_NAME).join(&pdf_rel));
                results.push((rel.clone(), Some(input_name), status, diff_link));
            }
        }
    }

    Ok(results)
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
fn build_report(mut results: Vec<DiffSummaryRow>) -> Result<String> {
    results.sort_unstable_by(
        |(left_template, left_input, ..), (right_template, right_input, ..)| {
            left_template
                .cmp(right_template)
                .then_with(|| left_input.cmp(right_input))
        },
    );

    let mut report = String::new();
    writeln!(report, "| Template | Input | Status |")?;
    writeln!(report, "| --- | --- | --- |")?;
    for (template, input_name, status, _) in results {
        writeln!(
            report,
            "| {} | {} | {} {status} |",
            template.display(),
            input_name.unwrap_or_default(),
            status_indicator(status),
        )?;
    }

    Ok(report)
}

/// Persist the generated Markdown report to disk.
async fn write_report(results_path: &Path, report: &str) -> Result<()> {
    tokio::fs::write(results_path, report)
        .await
        .with_context(|| format!("Failed to write {}", results_path.display()))
}

/// Execute the full PDF diff workflow from setup through summary generation.
async fn run() -> Result<()> {
    ensure_diff_pdf_is_installed()?;

    let project_dir = env::current_dir().context("Failed to get current directory")?;
    let paths = WorkspacePaths::new(project_dir);
    tracing::info!("Current directory: {}", paths.tmp_dir.display());

    prepare_workspace(&paths).await?;

    let groups = group_model_files(&paths.current_root, &paths.main_root)?;
    log_file_groups(&groups);

    let (current_context, main_context) = load_pdf_contexts(&paths).await?;

    let mut results = summarize_new_files(&paths, &current_context, groups.new_files).await?;
    results.extend(summarize_deleted_files(groups.deleted_files));
    results.extend(
        summarize_common_files(&paths, &current_context, &main_context, groups.common_files)
            .await?,
    );

    let report = build_report(results)?;
    write_report(&paths.results_path, &report).await?;
    info!("Wrote results to {}", paths.results_path.display());

    Ok(())
}

/// Compare PDFs rendered from the current and `main` model trees and summarize file changes.
#[tokio::main]
async fn main() -> Result<()> {
    init_tracing();
    run().await
}
