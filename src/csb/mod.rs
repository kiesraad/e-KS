//! CSB (Centraal Stembureau) domain.
//!
//! Mirrors the layout of the `app` domain but is scoped to the central
//! electoral council side of the workflow. For now it carries a single
//! placeholder `import` page plus its own request context, event, and
//! event-sourced store projection (see [`CsbStoreData`]).
pub mod audit_log;
pub mod examination;
pub mod import;
pub mod index;
pub mod monitoring;

mod context;
mod omission;
mod store_csb;
mod store_main;

pub use context::CsbContext;
pub use omission::{Omission, OmissionCategory, OmissionId, OmissionPlaceholders, OmissionType};
pub use store_csb::{CsbEvent, CsbStoreData};
#[cfg(any(test, feature = "dev-features"))]
pub use store_main::CsbMainEvent;
pub use store_main::{CSB_MAIN_STREAM_ID, CsbMainStoreData};
