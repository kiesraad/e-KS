use std::fmt::Write;

use sha2::{Digest, Sha256};

/// Lowercase hex SHA-256 of `data` (64 chars): the encoding shared by the
/// session token hash, the CSRF token hash, and the user-agent hash.
pub fn sha256_hex(data: &[u8]) -> String {
    let digest = Sha256::digest(data);
    let mut out = String::with_capacity(digest.len() * 2);
    for byte in digest {
        write!(out, "{byte:02x}").expect("writing to a String cannot fail");
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Matches the well-known SHA-256 test vector for "abc".
    #[test]
    fn sha256_hex_matches_known_vector() {
        assert_eq!(
            sha256_hex(b"abc"),
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
    }

    /// Output is always 64 lowercase hex characters.
    #[test]
    fn sha256_hex_is_64_lowercase_hex_chars() {
        let hex = sha256_hex(b"");
        assert_eq!(hex.len(), 64);
        assert!(
            hex.chars()
                .all(|c| c.is_ascii_hexdigit() && !c.is_ascii_uppercase())
        );
    }
}
