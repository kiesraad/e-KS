use rand::{RngExt, distr::Alphanumeric};
use serde::{Deserialize, Serialize};
use std::fmt::Display;

/// Opaque CSRF token value stored on the session and emitted in forms.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub struct TokenValue(pub String);

impl Display for TokenValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

/// Generate a fresh random CSRF token (24 chars of base62, ~142 bits).
pub fn generate_csrf_token() -> TokenValue {
    TokenValue(
        rand::rng()
            .sample_iter(&Alphanumeric)
            .take(24)
            .map(char::from)
            .collect(),
    )
}

pub trait WithCsrfToken: Default {
    fn with_csrf_token(self, csrf_token: TokenValue) -> Self;
}
