struct ParsedLine<'a> {
    indent: usize,
    key: &'a str,
    value: &'a str,
}

fn parse_line(line: &str) -> Option<ParsedLine<'_>> {
    let trimmed = line.trim();
    if trimmed.is_empty() || trimmed.starts_with('#') {
        return None;
    }

    let (key, value) = trimmed.split_once(':')?;
    let indent = line.chars().take_while(|c| c.is_whitespace()).count();
    let key = key.trim();
    let value = value.trim();

    if key.contains('.') {
        panic!("Keys containing '.' are not supported in locale YAML files");
    }
    if value == "|" || value == ">" {
        panic!("Multiline values are not supported in locale YAML files");
    }

    Some(ParsedLine { indent, key, value })
}

fn pop_to_indent(prefix: &mut Vec<String>, last_indent: usize, indent: usize) {
    if indent < last_indent {
        let levels_up = (last_indent - indent) / 2;
        for _ in 0..levels_up {
            prefix.pop();
        }
    }
}

/// Parses a minimal YAML mapping into flat `prefix.key` pairs.
///
/// This ignores blank lines and comments, expects indentation in 2-space
/// steps, and does not support complex YAML features.
pub(crate) fn naive_yaml_parse(prefix: &str, yml: &str) -> Vec<(String, String)> {
    let mut results = Vec::new();
    let mut prefix = vec![prefix.to_string()];
    let mut last_indent: usize = 0;

    for line in yml.trim_ascii().lines() {
        let Some(parsed) = parse_line(line) else {
            continue;
        };

        pop_to_indent(&mut prefix, last_indent, parsed.indent);

        if parsed.value.is_empty() {
            prefix.push(parsed.key.to_string());
        } else {
            let value = parsed
                .value
                .trim_matches('"')
                .trim_matches('\'')
                .to_string();
            results.push((format!("{}.{}", prefix.join("."), parsed.key), value));
        }

        last_indent = parsed.indent;
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

    #[test]
    fn test_skips_comments_and_blank_lines() {
        let yaml = r#"
# leading comment
greeting: "Hello"

nested:
  # inner comment
  value: "Nested"

goodnight: "Good night"
"#;

        let output = naive_yaml_parse("messages", yaml);

        assert_eq!(
            output,
            vec![
                ("messages.greeting".to_string(), "Hello".to_string()),
                ("messages.nested.value".to_string(), "Nested".to_string()),
                ("messages.goodnight".to_string(), "Good night".to_string()),
            ]
        );
    }

    #[test]
    fn test_skips_lines_without_a_colon() {
        let output = naive_yaml_parse("messages", "greeting: \"Hello\"\nnot a mapping line\n");

        assert_eq!(
            output,
            vec![("messages.greeting".to_string(), "Hello".to_string())]
        );
    }

    #[test]
    #[should_panic(expected = "Keys containing '.' are not supported")]
    fn test_rejects_dotted_keys() {
        naive_yaml_parse("messages", "greeting.formal: \"Hello\"\n");
    }

    #[test]
    #[should_panic(expected = "Multiline values are not supported")]
    fn test_rejects_literal_block_values() {
        naive_yaml_parse("messages", "greeting: |\n  Hello\n");
    }

    #[test]
    #[should_panic(expected = "Multiline values are not supported")]
    fn test_rejects_folded_block_values() {
        naive_yaml_parse("messages", "greeting: >\n  Hello\n");
    }
}
