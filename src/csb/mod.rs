//! CSB (Centraal Stembureau) domain.
//!
//! Mirrors the layout of the `app` domain but is scoped to the central
//! electoral council side of the workflow. For now it carries a single
//! placeholder `import` page plus its own request context, event, and
//! event-sourced store projection (see [`CsbStoreData`]).
pub mod examination;
pub mod import;
pub mod index;

mod context;
mod event;
mod omission;
mod store;

pub use context::CsbContext;
pub use event::CsbEvent;
pub use omission::{Omission, OmissionId};
pub use store::{CSB_MAIN_STREAM_ID, CsbMainEvent, CsbMainStoreData, CsbStoreData};
