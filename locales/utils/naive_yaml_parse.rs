/// Parses a minimal YAML mapping into flat `prefix.key` pairs.
///
/// This ignores blank lines and comments, expects indentation in 2-space
/// steps, and does not support complex YAML features.
fn naive_yaml_parse(prefix: &str, yml: &str) -> Vec<(String, String)> {
    let mut results = Vec::new();
    let lines = yml.trim_ascii().lines();
    let mut prefix = vec![prefix.to_string()];
    let mut last_indent: usize = 0;

    for line in lines {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }

        if let Some((key, value)) = trimmed.split_once(':') {
            let indent: usize = line.chars().take_while(|c| c.is_whitespace()).count();

            let key = key.trim();
            let value = value.trim();

            if key.contains('.') {
                panic!("Keys containing '.' are not supported in locale YAML files");
            }

            if value == "|" || value == ">" {
                panic!("Multiline values are not supported in locale YAML files");
            }

            if indent < last_indent {
                let levels_up = (last_indent - indent) / 2;
                for _ in 0..levels_up {
                    prefix.pop();
                }
            }

            if value.is_empty() {
                prefix.push(key.to_string());
            } else {
                let value = value.trim_matches('"').trim_matches('\'').to_string();

                results.push((format!("{}.{}", prefix.join("."), key), value));
            }

            last_indent = indent;
        }
    }

    results
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_naive_yaml_parse() {
        let yaml = r#"
greeting: "Hello"
farewell:
  morning: "Good morning"
  evening: Good evening
  formal:
    title: "Good evening, sir"
    closing: "Yours sincerely"
goodmorning:
  basic: "Good morning"
  polite: "Good morning to you"
goodnight: Good night
"#;

        let expected = vec![
            ("messages.greeting".to_string(), "Hello".to_string()),
            (
                "messages.farewell.morning".to_string(),
                "Good morning".to_string(),
            ),
            (
                "messages.farewell.evening".to_string(),
                "Good evening".to_string(),
            ),
            (
                "messages.farewell.formal.title".to_string(),
                "Good evening, sir".to_string(),
            ),
            (
                "messages.farewell.formal.closing".to_string(),
                "Yours sincerely".to_string(),
            ),
            (
                "messages.goodmorning.basic".to_string(),
                "Good morning".to_string(),
            ),
            (
                "messages.goodmorning.polite".to_string(),
                "Good morning to you".to_string(),
            ),
            ("messages.goodnight".to_string(), "Good night".to_string()),
        ];

        let output = naive_yaml_parse("messages", yaml);

        assert_eq!(output, expected);
    }
}
