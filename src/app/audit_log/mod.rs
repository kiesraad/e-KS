//! Audit log overview showing all applied events.
mod pages;
mod structs;

pub use pages::{AuditLogPath, router};
pub use structs::AuditLogEntry;
