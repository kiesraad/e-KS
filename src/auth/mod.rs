//! Authentication and session helpers.

/// BSN-based identifier derivation using HKDF-SHA256.
pub mod derive_id;
#[cfg(feature = "dev-features")]
pub mod dev_login;
/// Session model and token utilities.
pub mod session;
/// Postgres-backed session persistence (feature-gated).
#[cfg(feature = "database")]
mod session_db;
/// Session middleware and request extraction.
pub mod session_extractor;
/// Session storage with pluggable in-memory or Postgres backends.
pub mod session_store;
