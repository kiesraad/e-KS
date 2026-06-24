//! Authentication and session helpers.

/// BSN-based identifier derivation using HKDF-SHA256.
pub mod derive_id;

/// Authorization scope for sessions and streams.
pub mod scope;

/// Session model and token utilities.
pub mod session;

/// Session storage with pluggable in-memory or Postgres backends.
pub mod session_store;

/// Postgres-backed session persistence (feature-gated).
#[cfg(feature = "database")]
mod session_db;

/// Session middleware and request extraction.
pub mod session_extractor;

#[cfg(test)]
mod session_extractor_tests;

#[cfg(feature = "dev-features")]
pub mod dev_login;

#[cfg(all(feature = "dev-features", test))]
mod dev_login_tests;
