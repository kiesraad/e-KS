use serde::{Deserialize, Serialize};

use crate::{Event, StreamId, trans};

/// Events for the global CSB stream. Variants will be added as committee-wide
/// features are implemented (process steps, audit log, etc.).
#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum CsbMainEvent {
    DeveloperLogin { stream_id: StreamId },
}

impl Event for CsbMainEvent {
    fn category(&self) -> &'static str {
        match self {
            CsbMainEvent::DeveloperLogin { .. } => "system",
        }
    }

    fn key(&self) -> &'static str {
        match self {
            CsbMainEvent::DeveloperLogin { .. } => "developer_login",
        }
    }

    fn description(&self, locale: crate::Locale) -> String {
        match self {
            CsbMainEvent::DeveloperLogin { .. } => {
                trans!("audit_log.event.developer_login", locale)
            }
        }
    }

    fn details(&self) -> String {
        match self {
            CsbMainEvent::DeveloperLogin { .. } => "".to_string(),
        }
    }
}
