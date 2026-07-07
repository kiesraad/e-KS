use rand::{RngExt, distr::Alphanumeric};
use std::fmt::Display;
use subtle::ConstantTimeEq;

/// Raw CSRF token, stored on the session and emitted in forms.
#[derive(Clone, Debug, Eq, PartialEq, Default)]
pub struct TokenValue(pub String);

impl Display for TokenValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

/// Fresh random CSRF token (32 chars base62, ~190 bits), independent of the
/// session token.
pub fn generate_csrf_token() -> TokenValue {
    TokenValue(
        rand::rng()
            .sample_iter(&Alphanumeric)
            .take(32)
            .map(char::from)
            .collect(),
    )
}

/// Constant-time equality check of a submitted token against the expected one.
pub fn csrf_token_matches(submitted: &str, expected: &str) -> bool {
    submitted.as_bytes().ct_eq(expected.as_bytes()).into()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Freshly generated tokens are random base62 of the expected length.
    #[test]
    fn generate_produces_random_base62() {
        let a = generate_csrf_token();
        let b = generate_csrf_token();

        assert_eq!(a.0.len(), 32);
        assert!(a.0.chars().all(|c| c.is_ascii_alphanumeric()));
        assert_ne!(a, b); // overwhelmingly likely with ~190 bits
    }

    /// The constant-time check accepts only the expected token.
    #[test]
    fn matches_verifies_expected_token() {
        let raw = generate_csrf_token();

        assert!(csrf_token_matches(&raw.0, &raw.0));
        assert!(!csrf_token_matches("wrong", &raw.0));
        assert!(!csrf_token_matches(&raw.0[..16], &raw.0));
    }
}
