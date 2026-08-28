//! Shared view layer: the Askama filters and the request-scoped template
//! context that both web sections render through.
mod context;
pub mod filters;

pub use context::Context;
// Only the `pg` and `csb` guard tests read these; see `context.rs`.
#[cfg(test)]
pub(crate) use context::{CSB_PAPER_CORRECTIONS_STOP_PREFIX, DOWNLOAD_WARNING_PREFIXES};
