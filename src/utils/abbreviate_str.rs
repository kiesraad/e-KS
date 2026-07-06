/// Abbreviate a string to its first 8 characters (used for UUID previews).
pub fn abbreviate_str(s: &str) -> String {
    s[..8.min(s.len())].to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn abbreviate_str_short_string() {
        assert_eq!(abbreviate_str("abc"), "abc");
        assert_eq!(abbreviate_str(""), "");
    }

    #[test]
    fn abbreviate_str_long_string() {
        assert_eq!(abbreviate_str("123456789abcdef"), "12345678");
    }

    #[test]
    fn abbreviate_str_exactly_eight() {
        assert_eq!(abbreviate_str("12345678"), "12345678");
    }
}
