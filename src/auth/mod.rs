//! Authentication and session helpers.

/// BSN-based identifier derivation using HKDF-SHA256.
pub mod derive_id;

/// Session model and token utilities.
pub mod session;

/// Session storage with pluggable in-memory or Postgres backends.
pub mod session_store;

/// Postgres-backed session persistence (feature-gated).
#[cfg(feature = "database")]
mod session_db;

/// Pending AuthnRequest ID storage with pluggable in-memory or Postgres backends.
pub mod pending_request_store;

/// Postgres-backed pending-request persistence (feature-gated).
#[cfg(feature = "database")]
mod pending_request_db;

/// Session cookie helpers and request extraction.
pub mod session_extractor;
