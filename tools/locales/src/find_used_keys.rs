use std::collections::BTreeSet;

/// Recursively collects files with the given extension under `dir`.
fn collect_files_recursively(
    dir: &std::path::Path,
    extension: &str,
    files: &mut Vec<std::path::PathBuf>,
) {
    let entries = std::fs::read_dir(dir).expect("Failed to read source directory");
    for entry in entries {
        let entry = entry.expect("Failed to read source directory entry");
        let path = entry.path();
        if path.is_dir() {
            collect_files_recursively(&path, extension, files);
        } else if path.extension().and_then(|ext| ext.to_str()) == Some(extension) {
            files.push(path);
        }
    }
}

/// Adds the first capture group of every `re` match in `files` to `keys`.
fn collect_captured_keys(
    keys: &mut BTreeSet<String>,
    re: &regex::Regex,
    files: &[std::path::PathBuf],
) {
    for file in files {
        let haystack = std::fs::read_to_string(file).expect("Failed to read source file");

        for capture in re.captures_iter(&haystack) {
            if let Some(key) = capture.get(1) {
                keys.insert(key.as_str().to_string());
            }
        }
    }
}

/// Finds translation keys used in templates and Rust sources under `path`.
///
/// Keys are collected using simple regex scans, deduplicated and sorted.
pub fn find_used_keys(path: &std::path::Path) -> Vec<String> {
    let mut used_keys = BTreeSet::new();

    let template_re = regex::Regex::new(r#""([\w\.]+)"\|trans"#).expect("Invalid template regex");
    let mut template_files = Vec::new();
    // Scan every template root listed in `askama.toml` (`src/pg` and `src/csb`).
    for templates_dir in ["pg", "csb"] {
        let templates_dir = path.join("src").join(templates_dir);
        collect_files_recursively(&templates_dir, "html", &mut template_files);
    }
    collect_captured_keys(&mut used_keys, &template_re, &template_files);

    let source_re =
        regex::Regex::new(r#"trans!\s*\(\s*"([\w\.]+)""#).expect("Invalid source regex");
    let mut source_files = Vec::new();
    collect_files_recursively(&path.join("src"), "rs", &mut source_files);
    collect_captured_keys(&mut used_keys, &source_re, &source_files);

    used_keys.into_iter().collect()
}
