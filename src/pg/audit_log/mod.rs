//! Audit log overview showing all applied events.
mod event_info;
mod pages;
mod paths;
mod structs;

pub use pages::router;
pub use paths::{AuditLogDetailPath, AuditLogPath};
pub use structs::{AuditLogDetail, AuditLogEntry};
