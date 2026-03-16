/// Collects top-level `.yml` files from `dir`.
///
/// Returns a sorted list and panics if the directory cannot be read or if no
/// locale files are found.
pub fn collect_locale_files(dir: &std::path::Path) -> Vec<std::path::PathBuf> {
    let mut locale_files = Vec::new();

    if dir.is_dir() {
        for entry in std::fs::read_dir(dir).expect("Failed to read locale directory") {
            let entry = entry.expect("Failed to read locale directory entry");
            let path = entry.path();

            if path.extension().and_then(|ext| ext.to_str()) == Some("yml") {
                locale_files.push(path);
            }
        }
    }

    if locale_files.is_empty() {
        panic!("No locale files found in '{}'", dir.display());
    }

    locale_files.sort();

    locale_files
}
