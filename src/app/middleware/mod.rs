//! HTTP middleware and infrastructure endpoints that need [`AppState`](crate::AppState).
//!
//! These live under `app/` (rather than `core/` or `utils/`) because they
//! depend on [`AppState`](crate::AppState)/[`Config`](crate::Config); keeping
//! them here avoids cycles with the otherwise-leaf `utils` and `core` modules.

pub mod eks_key;
pub mod health;

#[cfg(any(feature = "dev-features", not(feature = "memory-serve")))]
pub mod proxy;

#[cfg(test)]
mod health_tests;
