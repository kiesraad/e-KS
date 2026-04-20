//! Audit log overview showing all applied events.
mod pages;
mod structs;

pub use pages::{AuditLogDetailPath, AuditLogPath, router};
pub use structs::{AuditLogDetail, AuditLogEntry, abbreviate_str};
