use rand::{RngExt, distr::Alphanumeric};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fmt::Display;
use subtle::ConstantTimeEq;

/// Raw CSRF token, emitted in forms and carried in the CSRF cookie. Only its
/// hash is stored server-side.
#[derive(Clone, Debug, Eq, PartialEq, Default, Serialize, Deserialize)]
pub struct TokenValue(pub String);

impl Display for TokenValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

/// Domain-separation prefix for the CSRF hash.
const CSRF_DOMAIN_PREFIX: &[u8] = b"eks-csrf-v1:";

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

/// Prefixed hex SHA-256 of a raw CSRF token: the only CSRF material at rest.
pub fn hash_csrf_token(raw: &str) -> String {
    let digest = Sha256::digest([CSRF_DOMAIN_PREFIX, raw.as_bytes()].concat());
    digest.iter().map(|b| format!("{b:02x}")).collect()
}

/// Constant-time check that `submitted` hashes to `stored_hash`.
pub fn csrf_token_matches(submitted: &str, stored_hash: &str) -> bool {
    hash_csrf_token(submitted)
        .as_bytes()
        .ct_eq(stored_hash.as_bytes())
        .into()
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

    /// The stored hash is deterministic, domain-separated from a plain SHA-256,
    /// and 32 bytes hex-encoded.
    #[test]
    fn hash_is_deterministic_and_domain_separated() {
        let raw = "csrf-token-abc";
        let hash = hash_csrf_token(raw);

        assert_eq!(hash, hash_csrf_token(raw));
        assert_eq!(hash.len(), 64); // 32 bytes hex-encoded

        let plain_hash: String = Sha256::digest(raw.as_bytes())
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect();
        assert_ne!(hash, plain_hash);
    }

    /// Distinct raw tokens hash to distinct values.
    #[test]
    fn hash_differs_per_token() {
        assert_ne!(hash_csrf_token("token-a"), hash_csrf_token("token-b"));
    }

    /// The constant-time check accepts only the token behind a stored hash.
    #[test]
    fn matches_verifies_against_stored_hash() {
        let raw = generate_csrf_token();
        let stored = hash_csrf_token(&raw.0);

        assert!(csrf_token_matches(&raw.0, &stored));
        assert!(!csrf_token_matches("wrong", &stored));
        assert!(!csrf_token_matches(&raw.0, "not-a-hash"));
    }
}
