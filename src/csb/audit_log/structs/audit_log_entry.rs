use chrono::{DateTime, Utc};

use crate::{CsbEvent, CsbMainEvent, Locale, StreamId, store::StoreEvent, trans};

/// A single row in the CSB audit log, covering events from both the global
/// main stream and per-import political-group streams.
pub struct CsbAuditLogEntry {
    pub event_id: usize,
    pub stream_id: StreamId,
    /// Human-readable label for the source stream
    pub stream_label: String,
    pub event_category: &'static str,
    pub event_key: &'static str,
    pub description: String,
    pub created_at: DateTime<Utc>,
}

impl CsbAuditLogEntry {
    pub fn from_main_event(
        event: StoreEvent<CsbMainEvent>,
        stream_id: StreamId,
        locale: Locale,
    ) -> Self {
        Self {
            event_id: event.event_id,
            stream_id,
            stream_label: trans!("audit_log.filter.csb_main_stream", locale),
            event_category: event.payload.event_category(),
            event_key: event.payload.event_key(),
            description: csb_main_event_description(&event.payload, locale),
            created_at: event.created_at,
        }
    }

    pub fn from_import_event(
        event: StoreEvent<CsbEvent>,
        stream_id: StreamId,
        stream_label: String,
        locale: Locale,
    ) -> Self {
        Self {
            event_id: event.event_id,
            stream_id,
            stream_label,
            event_category: event.payload.event_category(),
            event_key: event.payload.event_key(),
            description: csb_event_description(&event.payload, locale),
            created_at: event.created_at,
        }
    }

    pub fn matches_search(&self, query: &str) -> bool {
        let query = query.to_lowercase();
        self.description.to_lowercase().contains(&query)
            || self.stream_label.to_lowercase().contains(&query)
    }

    pub fn detail_path(&self) -> String {
        format!("/csb/audit-log/{}/{}", self.stream_id, self.event_id)
    }
}

pub fn csb_main_event_description(event: &CsbMainEvent, locale: Locale) -> String {
    match event {
        CsbMainEvent::DeveloperLogin { .. } => trans!("audit_log.event.developer_login", locale),
    }
}

pub fn csb_event_description(event: &CsbEvent, locale: Locale) -> String {
    match event {
        CsbEvent::Import { .. } => trans!("audit_log.event.import", locale),
        CsbEvent::SetFinished(_) => trans!("audit_log.event.set_finished", locale),
        CsbEvent::CreateOmission(_) => trans!("audit_log.event.create_omission", locale),
        CsbEvent::UpdateOmission(_) => trans!("audit_log.event.update_omission", locale),
        CsbEvent::DeleteOmission { .. } => trans!("audit_log.event.delete_omission", locale),
    }
}
