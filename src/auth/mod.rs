//! Authentication and session helpers.

/// BSN-based identifier derivation using HKDF-SHA256.
pub mod derive_id;
#[cfg(feature = "dev-features")]
pub mod dev_login;
/// Session model and token utilities.
pub mod session;
/// Session middleware and request extraction.
pub mod session_extractor;
/// Session storage backed by an in-memory map.
pub mod session_store;
