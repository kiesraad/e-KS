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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::temp_dir;

    fn write(dir: &std::path::Path, name: &str) {
        std::fs::write(dir.join(name), "key: value\n").expect("write file");
    }

    #[test]
    fn collects_yml_files_sorted() {
        let dir = temp_dir();
        write(&dir, "session.yml");
        write(&dir, "audit_log.yml");

        let files = collect_locale_files(&dir);

        let names: Vec<_> = files
            .iter()
            .map(|path| path.file_name().unwrap().to_str().unwrap())
            .collect();
        assert_eq!(names, ["audit_log.yml", "session.yml"]);
    }

    #[test]
    fn ignores_other_extensions_and_subdirectories() {
        let dir = temp_dir();
        write(&dir, "audit_log.yml");
        write(&dir, "notes.yaml");
        write(&dir, "README.md");
        let nested = dir.join("nested");
        std::fs::create_dir_all(&nested).expect("create nested dir");
        write(&nested, "nested.yml");

        let files = collect_locale_files(&dir);

        let names: Vec<_> = files
            .iter()
            .map(|path| path.file_name().unwrap().to_str().unwrap())
            .collect();
        assert_eq!(names, ["audit_log.yml"]);
    }

    #[test]
    #[should_panic(expected = "No locale files found")]
    fn panics_on_directory_without_locale_files() {
        collect_locale_files(&temp_dir());
    }

    #[test]
    #[should_panic(expected = "No locale files found")]
    fn panics_on_missing_directory() {
        collect_locale_files(&temp_dir().join("missing"));
    }
}
