//! HTTP middleware and infrastructure endpoints that need [`AppState`](crate::AppState).
//!
//! These live under `app/` (rather than `core/` or `utils/`) because they
//! depend on [`AppState`](crate::AppState)/[`Config`](crate::Config); keeping
//! them here avoids cycles with the otherwise-leaf `utils` and `core` modules.

pub mod eks_key;
pub mod health;
pub mod maintenance;
pub mod session;

#[cfg(not(feature = "memory-serve"))]
pub mod proxy;

#[cfg(feature = "dev-features")]
pub mod dev_login;

#[cfg(test)]
mod health_tests;

#[cfg(test)]
mod session_tests;

#[cfg(all(feature = "dev-features", test))]
mod dev_login_tests;
