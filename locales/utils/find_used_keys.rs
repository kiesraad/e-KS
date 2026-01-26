/// Recursively collects files with the given extension under `dir`.
fn collect_files_recursively(
    dir: &std::path::Path,
    extension: &str,
    files: &mut Vec<std::path::PathBuf>,
) {
    for entry in std::fs::read_dir(dir).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        if path.is_dir() {
            collect_files_recursively(&path, extension, files);
        } else if path.extension().and_then(|ext| ext.to_str()) == Some(extension) {
            files.push(path);
        }
    }
}

/// Finds translation keys used in templates and Rust sources under `path`.
///
/// Keys are collected using simple regex scans and deduplicated
fn find_used_keys(path: &std::path::Path) -> Vec<String> {
    let mut used_keys = Vec::new();

    let re = regex::Regex::new(r#""([\w\.]+)"\|trans"#).unwrap();
    let templates_dir = path.join("templates");
    let mut template_files = Vec::new();
    collect_files_recursively(&templates_dir, "html", &mut template_files);

    for template_file in template_files {
        let haystack = std::fs::read_to_string(&template_file).unwrap();

        for capture in re.captures_iter(&haystack) {
            let key = capture.get(1).unwrap().as_str();

            if used_keys.contains(&key.to_string()) {
                continue;
            }

            used_keys.push(key.to_string());
        }
    }

    let re = regex::Regex::new(r#"trans!\("([\w\.]+)""#).unwrap();
    let sources_dir = path.join("src");
    let mut source_files = Vec::new();
    collect_files_recursively(&sources_dir, "rs", &mut source_files);

    for source_file in source_files {
        let haystack = std::fs::read_to_string(&source_file).unwrap();

        for capture in re.captures_iter(&haystack) {
            let key = capture.get(1).unwrap().as_str();

            if used_keys.contains(&key.to_string()) {
                continue;
            }

            used_keys.push(key.to_string());
        }
    }

    used_keys.sort();

    used_keys
}
