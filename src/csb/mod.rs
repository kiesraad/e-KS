//! CSB (Centraal Stembureau) domain.
//!
//! Mirrors the layout of the `pg` domain but is scoped to the central electoral
//! council side of the workflow: the import and examination pages plus their own
//! request context. The events and store projections these pages read from live
//! in [`crate::projection`].
pub mod audit_log;
pub mod examination;
pub mod import;
pub mod index;
pub mod login;
pub mod monitoring;

mod context;

pub use context::CsbContext;
