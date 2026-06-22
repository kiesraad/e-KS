//! CSB (Centraal Stembureau) domain.
//!
//! Mirrors the layout of the `app` domain but is scoped to the central
//! electoral council side of the workflow. For now it carries a single
//! placeholder `import` page plus its own request context, event, and
//! event-sourced store projection (see [`CsbStoreData`]).
pub mod examination;
pub mod import;

mod context;
mod event;
mod store;

pub use context::CsbContext;
pub use event::CsbEvent;
pub use store::CsbStoreData;
