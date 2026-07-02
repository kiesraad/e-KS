use serde::{Deserialize, Serialize};

use crate::StreamId;

/// Events for the global CSB stream. Variants will be added as committee-wide
/// features are implemented (process steps, audit log, etc.).
#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum CsbMainEvent {
    DeveloperLogin { stream_id: StreamId },
}

impl CsbMainEvent {
    pub fn event_category(&self) -> &'static str {
        match self {
            CsbMainEvent::DeveloperLogin { .. } => "system",
        }
    }

    pub fn event_key(&self) -> &'static str {
        match self {
            CsbMainEvent::DeveloperLogin { .. } => "developer_login",
        }
    }
}
