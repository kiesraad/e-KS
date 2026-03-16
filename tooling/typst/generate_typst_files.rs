use std::{
    collections::BTreeSet,
    env, fs, io,
    path::{Path, PathBuf},
};

/// Generates the `typst_files.rs` include used by the embedded Typst server.
///
/// This is called from `build.rs` when the `embed-typst` feature is enabled.
/// It also registers all files under `models/` as build inputs so Cargo reruns
/// the build script whenever Typst sources, fonts, or related assets change.
pub fn generate_typst_files_during_build(out_dir: &str) -> io::Result<()> {
    emit_rerun_if_changed(Path::new("models"));
    generate_typst_files(out_dir)?;

    Ok(())
}

fn generate_typst_files(out_dir: &str) -> io::Result<()> {
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR"));
    let models_dir = manifest_dir.join("models");
    let output_path = Path::new(out_dir).join("typst_files.rs");

    let mut assets = Vec::new();
    collect_typst_assets(&models_dir, &manifest_dir, &mut assets)?;
    // Keep generation stable so rebuilds only change when the asset set changes.
    assets.sort_unstable_by(|left, right| left.0.cmp(&right.0).then_with(|| left.1.cmp(&right.1)));

    let mut seen = BTreeSet::new();
    for (name, _) in &assets {
        assert!(
            seen.insert(name.clone()),
            // Typst looks up embedded assets by file name, so two different paths
            // cannot share the same basename.
            "duplicate embedded Typst asset name: {name}",
        );
    }

    // Generate a compact Rust expression that `include!` can embed directly.
    let mut generated = String::from("&[\n");
    for (name, relative_path) in assets {
        generated.push_str("    (\n");
        generated.push_str(&format!("        {:?},\n", name));
        generated.push_str(&format!(
            "        include_bytes!(concat!(env!(\"CARGO_MANIFEST_DIR\"), {:?})),\n",
            format!("/{}", relative_path)
        ));
        generated.push_str("    ),\n");
    }
    generated.push_str("]\n");

    fs::write(output_path, generated)
}

fn collect_typst_assets(
    dir: &Path,
    manifest_dir: &Path,
    assets: &mut Vec<(String, String)>,
) -> io::Result<()> {
    // Sort directory traversal for deterministic output across filesystems.
    let mut entries = fs::read_dir(dir)?.collect::<Result<Vec<_>, _>>()?;
    entries.sort_unstable_by_key(|entry| entry.path());

    for entry in entries {
        let path = entry.path();
        if path.is_dir() {
            collect_typst_assets(&path, manifest_dir, assets)?;
            continue;
        }

        if !is_embedded_typst_asset(&path) {
            continue;
        }

        let name = path
            .file_name()
            .and_then(|value| value.to_str())
            .expect("asset name should be valid UTF-8")
            .to_owned();
        let relative_path = path
            .strip_prefix(manifest_dir)
            .expect("asset should live under the manifest directory")
            .to_str()
            .expect("asset path should be valid UTF-8")
            .replace(std::path::MAIN_SEPARATOR, "/");

        assets.push((name, relative_path));
    }

    Ok(())
}

fn emit_rerun_if_changed(path: &Path) {
    println!("cargo::rerun-if-changed={}", path.display());
    // Cargo accepts directory paths here and scans them recursively for changes,
    // so a single directive for `models/` is enough.
}

fn is_embedded_typst_asset(path: &Path) -> bool {
    // Keep this in sync with the asset types the embedded Typst runtime expects.
    matches!(
        path.extension().and_then(|value| value.to_str()),
        Some("typ" | "ttf" | "otf")
    )
}
