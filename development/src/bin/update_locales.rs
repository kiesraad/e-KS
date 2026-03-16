use std::{
    borrow::Cow,
    collections::{BTreeMap, BTreeSet, HashSet},
    path::Path,
};

use anyhow::{Context, Result};
use saphyr::{LoadableYamlNode, Mapping, Scalar, Yaml, YamlEmitter};

/// In-memory representation of locale YAML structures.
#[derive(Debug, Clone)]
enum LocaleNode {
    Map(BTreeMap<String, LocaleNode>),
    String(String),
}

include!("../../../tooling/locales/collect_locale_files.rs");
include!("../../../tooling/locales/find_used_keys.rs");

/// Parse YAML into a structured tree while enforcing string-only leaves.
fn yaml_to_node(yaml: &Yaml, file: &Path, path: &str) -> Result<LocaleNode> {
    let display_path = if path.is_empty() { "<root>" } else { path };

    match yaml {
        Yaml::Mapping(mapping) => {
            let mut map = BTreeMap::new();

            for (key, value) in mapping {
                let key = key.as_str().with_context(|| {
                    format!("non-string key in {} at {}", file.display(), display_path)
                })?;

                let child_path = if path.is_empty() {
                    key.to_string()
                } else {
                    format!("{path}.{key}")
                };

                let node = yaml_to_node(value, file, &child_path)?;
                map.insert(key.to_string(), node);
            }

            Ok(LocaleNode::Map(map))
        }
        Yaml::Sequence(_) => anyhow::bail!(
            "arrays are not allowed in {} at {}",
            file.display(),
            display_path
        ),
        Yaml::Value(_) => {
            if let Some(value) = yaml.as_str() {
                Ok(LocaleNode::String(value.to_string()))
            } else {
                anyhow::bail!("non-string value in {} at {}", file.display(), display_path)
            }
        }
        Yaml::Tagged(_, inner) => yaml_to_node(inner, file, path),
        Yaml::Alias(_) | Yaml::BadValue | Yaml::Representation(_, _, _) => {
            anyhow::bail!(
                "unsupported value in {} at {}",
                file.display(),
                display_path
            )
        }
    }
}

/// Prune any keys that are not present in the used set.
fn retain_used(node: &mut LocaleNode, used: &HashSet<String>, prefix: &str) -> bool {
    match node {
        LocaleNode::String(_) => used.contains(prefix),
        LocaleNode::Map(map) => {
            let keys: Vec<String> = map.keys().cloned().collect();

            for key in keys {
                let child_prefix = if prefix.is_empty() {
                    key.clone()
                } else {
                    format!("{prefix}.{key}")
                };

                let keep = map
                    .get_mut(&key)
                    .map(|child| retain_used(child, used, &child_prefix))
                    .unwrap_or(false);

                if !keep {
                    map.remove(&key);
                }
            }

            !map.is_empty()
        }
    }
}

/// Ensure the full key path exists, inserting the key value for new leaf values.
fn insert_used_path(node: &mut LocaleNode, path: &str, file: &Path) -> Result<()> {
    if path.is_empty() {
        return Ok(());
    }

    let segments: Vec<&str> = path.split('.').collect();

    fn insert_segments(
        node: &mut LocaleNode,
        segments: &[&str],
        full_path: &str,
        file: &Path,
    ) -> Result<()> {
        match node {
            LocaleNode::Map(map) => {
                let key = segments[0];

                if segments.len() == 1 {
                    match map.entry(key.to_string()) {
                        std::collections::btree_map::Entry::Vacant(entry) => {
                            entry.insert(LocaleNode::String(key.to_string()));
                        }
                        std::collections::btree_map::Entry::Occupied(entry) => {
                            if matches!(entry.get(), LocaleNode::Map(_)) {
                                anyhow::bail!(
                                    "expected string for {} in {}",
                                    full_path,
                                    file.display()
                                );
                            }
                        }
                    }

                    return Ok(());
                }

                let entry = map
                    .entry(key.to_string())
                    .or_insert_with(|| LocaleNode::Map(BTreeMap::new()));

                match entry {
                    LocaleNode::Map(_) => {
                        insert_segments(entry, &segments[1..], full_path, file)?;
                    }
                    LocaleNode::String(_) => {
                        anyhow::bail!("expected mapping for {} in {}", full_path, file.display());
                    }
                }
            }
            LocaleNode::String(_) => {
                anyhow::bail!("expected mapping for {} in {}", full_path, file.display());
            }
        }

        Ok(())
    }

    insert_segments(node, &segments, path, file)
}

/// Convert a LocaleNode tree back into a YAML value.
fn node_to_yaml(node: &LocaleNode) -> Yaml<'static> {
    match node {
        LocaleNode::String(value) => Yaml::Value(Scalar::String(Cow::Owned(value.clone()))),
        LocaleNode::Map(map) => {
            let mut mapping: Mapping = Mapping::new();

            for (key, value) in map {
                let key_yaml = Yaml::Value(Scalar::String(Cow::Owned(key.clone())));
                let value_yaml = node_to_yaml(value);
                mapping.insert(key_yaml, value_yaml);
            }

            Yaml::Mapping(mapping)
        }
    }
}

/// Collect leaf paths from a locale tree so we can diff against used keys.
fn collect_leaf_paths(node: &LocaleNode) -> BTreeSet<String> {
    fn walk(node: &LocaleNode, prefix: &str, out: &mut BTreeSet<String>) {
        match node {
            LocaleNode::String(_) => {
                if !prefix.is_empty() {
                    out.insert(prefix.to_string());
                }
            }
            LocaleNode::Map(map) => {
                for (key, value) in map {
                    let child_prefix = if prefix.is_empty() {
                        key.clone()
                    } else {
                        format!("{prefix}.{key}")
                    };
                    walk(value, &child_prefix, out);
                }
            }
        }
    }

    let mut out = BTreeSet::new();
    walk(node, "", &mut out);
    out
}

fn main() -> Result<()> {
    let used_keys = find_used_keys(Path::new("."));
    let mut files_processed = 0usize;
    let mut files_changed = 0usize;
    let mut total_added = 0usize;
    let mut total_removed = 0usize;

    for lang in &["en", "nl"] {
        let locale_dir = Path::new("locales").join(lang);
        let locale_files = collect_locale_files(&locale_dir);

        for file in locale_files {
            files_processed += 1;
            let basename = file
                .file_stem()
                .and_then(|s| s.to_str())
                .context("Failed to get locale file stem")?;

            let input = std::fs::read_to_string(&file)
                .with_context(|| format!("failed to read locale file {}", file.display()))?;

            let docs = Yaml::load_from_str(&input)
                .with_context(|| format!("failed to parse YAML in {}", file.display()))?;

            if docs.len() != 1 {
                anyhow::bail!(
                    "expected exactly one YAML document in {}, found {}",
                    file.display(),
                    docs.len()
                );
            }

            let root = &docs[0];

            let mut node = match yaml_to_node(root, &file, "")? {
                LocaleNode::Map(map) => LocaleNode::Map(map),
                LocaleNode::String(_) => {
                    anyhow::bail!("expected mapping at root of {}", file.display());
                }
            };

            let existing_keys = collect_leaf_paths(&node);
            let prefix = format!("{basename}.");
            let used_set: HashSet<String> = used_keys
                .iter()
                .filter_map(|key| {
                    if key.starts_with(&prefix) {
                        Some(key[prefix.len()..].to_string())
                    } else {
                        None
                    }
                })
                .collect();

            let mut added_keys = BTreeSet::new();
            for used_key in &used_set {
                if !existing_keys.contains(used_key) {
                    added_keys.insert(used_key.clone());
                }
            }

            let mut removed_keys = BTreeSet::new();
            for existing_key in &existing_keys {
                if !used_set.contains(existing_key) {
                    removed_keys.insert(existing_key.clone());
                }
            }

            if !added_keys.is_empty() || !removed_keys.is_empty() {
                files_changed += 1;
            }

            for removed_key in &removed_keys {
                let full_key = if removed_key.is_empty() {
                    basename.to_string()
                } else {
                    format!("{basename}.{removed_key}")
                };
                println!("remove {} ({})", full_key, file.display());
            }

            for added_key in &added_keys {
                let full_key = if added_key.is_empty() {
                    basename.to_string()
                } else {
                    format!("{basename}.{added_key}")
                };
                println!("add {} ({})", full_key, file.display());
            }

            total_added += added_keys.len();
            total_removed += removed_keys.len();

            retain_used(&mut node, &used_set, "");

            for used_key in &used_set {
                insert_used_path(&mut node, used_key, &file)?;
            }

            let yaml_out = node_to_yaml(&node);
            let mut output = String::new();
            YamlEmitter::new(&mut output)
                .dump(&yaml_out)
                .with_context(|| format!("failed to emit YAML for {}", file.display()))?;

            let mut output = output.strip_prefix("---\n").unwrap_or(&output).to_string();
            output.push('\n');

            std::fs::write(&file, output)
                .with_context(|| format!("failed to write locale file {}", file.display()))?;
        }
    }

    println!(
        "Finished processing locale files: processed {} file(s), {} changed, {} added, {} removed.",
        files_processed, files_changed, total_added, total_removed
    );

    Ok(())
}
