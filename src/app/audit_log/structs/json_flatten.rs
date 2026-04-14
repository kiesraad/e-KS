//! Flatten a JSON value into a map of dot-notation paths → string values.
//!
//! Scalar arrays collapse into a single comma-joined entry (so reorder-agnostic
//! lists like `electoral_districts` produce one diff row), except when the
//! array key is in `POSITIONAL_ARRAY_KEYS` — those keep per-index entries so
//! reorders stay visible.

use std::collections::BTreeMap;

/// Scalar-array fields where position is semantically meaningful.
const POSITIONAL_ARRAY_KEYS: &[&str] = &["candidates"];

fn is_scalar(value: &serde_json::Value) -> bool {
    matches!(
        value,
        serde_json::Value::String(_)
            | serde_json::Value::Number(_)
            | serde_json::Value::Bool(_)
            | serde_json::Value::Null
    )
}

fn scalar_to_string(value: &serde_json::Value) -> String {
    match value {
        serde_json::Value::String(s) => s.clone(),
        serde_json::Value::Number(n) => n.to_string(),
        serde_json::Value::Bool(b) => b.to_string(),
        serde_json::Value::Null => String::new(),
        _ => String::new(),
    }
}

fn join_scalars(arr: &[serde_json::Value]) -> String {
    arr.iter()
        .map(scalar_to_string)
        .collect::<Vec<_>>()
        .join(", ")
}

fn is_positional_array_key(prefix: &str) -> bool {
    let leaf = prefix.rsplit('.').next().unwrap_or(prefix);
    POSITIONAL_ARRAY_KEYS.contains(&leaf)
}

/// Recursively flatten a JSON value into dot-notation key-value pairs.
pub(super) fn flatten(value: &serde_json::Value, prefix: &str) -> BTreeMap<String, String> {
    let mut map = BTreeMap::new();

    match value {
        serde_json::Value::Object(obj) => {
            for (key, val) in obj {
                let full_key = if prefix.is_empty() {
                    key.clone()
                } else {
                    format!("{prefix}.{key}")
                };
                map.extend(flatten(val, &full_key));
            }
        }
        serde_json::Value::Array(arr) => {
            let positional = is_positional_array_key(prefix);
            if !prefix.is_empty() && arr.iter().all(is_scalar) && !positional {
                map.insert(prefix.to_string(), join_scalars(arr));
            } else {
                for (i, val) in arr.iter().enumerate() {
                    let full_key = if prefix.is_empty() {
                        i.to_string()
                    } else {
                        format!("{prefix}.{i}")
                    };
                    map.extend(flatten(val, &full_key));
                }
            }
        }
        serde_json::Value::Null => {
            if !prefix.is_empty() {
                map.insert(prefix.to_string(), String::new());
            }
        }
        serde_json::Value::Bool(b) => {
            if !prefix.is_empty() {
                map.insert(prefix.to_string(), b.to_string());
            }
        }
        serde_json::Value::Number(n) => {
            if !prefix.is_empty() {
                map.insert(prefix.to_string(), n.to_string());
            }
        }
        serde_json::Value::String(s) => {
            if !prefix.is_empty() {
                map.insert(prefix.to_string(), s.clone());
            }
        }
    }

    map
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn flatten_simple_object() {
        let val = serde_json::json!({
            "name": "Alice",
            "age": 30
        });
        let flat = flatten(&val, "");
        assert_eq!(flat.get("name").unwrap(), "Alice");
        assert_eq!(flat.get("age").unwrap(), "30");
    }

    #[test]
    fn flatten_nested_object() {
        let val = serde_json::json!({
            "name": {
                "first": "Alice",
                "last": "Smith"
            }
        });
        let flat = flatten(&val, "");
        assert_eq!(flat.get("name.first").unwrap(), "Alice");
        assert_eq!(flat.get("name.last").unwrap(), "Smith");
    }

    #[test]
    fn flatten_array_of_scalars_joins_as_csv() {
        let val = serde_json::json!({
            "items": ["a", "b", "c"]
        });
        let flat = flatten(&val, "");
        assert_eq!(flat.get("items").unwrap(), "a, b, c");
        assert!(!flat.contains_key("items.0"));
    }

    #[test]
    fn flatten_array_of_objects_uses_index_paths() {
        let val = serde_json::json!({
            "items": [{"name": "a"}, {"name": "b"}]
        });
        let flat = flatten(&val, "");
        assert_eq!(flat.get("items.0.name").unwrap(), "a");
        assert_eq!(flat.get("items.1.name").unwrap(), "b");
        assert!(!flat.contains_key("items"));
    }

    #[test]
    fn flatten_positional_array_keeps_index_paths() {
        let val = serde_json::json!({
            "candidates": ["a", "b"]
        });
        let flat = flatten(&val, "");
        assert_eq!(flat.get("candidates.0").unwrap(), "a");
        assert_eq!(flat.get("candidates.1").unwrap(), "b");
        assert!(!flat.contains_key("candidates"));
    }

    #[test]
    fn flatten_null_value() {
        let val = serde_json::json!({ "field": null });
        let flat = flatten(&val, "");
        assert_eq!(flat.get("field").unwrap(), "");
    }

    #[test]
    fn flatten_boolean_value() {
        let val = serde_json::json!({
            "active": true,
            "deleted": false
        });
        let flat = flatten(&val, "");
        assert_eq!(flat.get("active").unwrap(), "true");
        assert_eq!(flat.get("deleted").unwrap(), "false");
    }

    #[test]
    fn flatten_deeply_nested() {
        let val = serde_json::json!({
            "a": { "b": { "c": "deep" } }
        });
        let flat = flatten(&val, "");
        assert_eq!(flat.get("a.b.c").unwrap(), "deep");
        assert_eq!(flat.len(), 1);
    }

    #[test]
    fn flatten_empty_object() {
        let val = serde_json::json!({});
        let flat = flatten(&val, "");
        assert!(flat.is_empty());
    }

    #[test]
    fn flatten_empty_array() {
        let val = serde_json::json!({ "items": [] });
        let flat = flatten(&val, "");
        assert_eq!(flat.get("items").unwrap(), "");
    }

    #[test]
    fn flatten_mixed_types() {
        let val = serde_json::json!({
            "name": "test",
            "count": 42,
            "active": true,
            "tags": ["a"],
            "meta": null
        });
        let flat = flatten(&val, "");
        assert_eq!(flat.get("name").unwrap(), "test");
        assert_eq!(flat.get("count").unwrap(), "42");
        assert_eq!(flat.get("active").unwrap(), "true");
        assert_eq!(flat.get("tags").unwrap(), "a");
        assert_eq!(flat.get("meta").unwrap(), "");
    }

    #[test]
    fn flatten_root_scalar_ignored() {
        let val = serde_json::json!("hello");
        let flat = flatten(&val, "");
        assert!(flat.is_empty());
    }

    #[test]
    fn flatten_with_prefix() {
        let val = serde_json::json!({ "x": 1 });
        let flat = flatten(&val, "root");
        assert_eq!(flat.get("root.x").unwrap(), "1");
    }
}
