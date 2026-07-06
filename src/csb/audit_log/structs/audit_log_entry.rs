use chrono::{DateTime, Utc};

use crate::{CsbEvent, CsbMainEvent, Event, Locale, StreamId, store::StoreEvent, trans};

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
            event_category: event.payload.category(),
            event_key: event.payload.key(),
            description: event.payload.description(locale),
            created_at: event.created_at,
        }
    }

    pub fn from_event(
        event: StoreEvent<CsbEvent>,
        stream_id: StreamId,
        stream_label: String,
        locale: Locale,
    ) -> Self {
        Self {
            event_id: event.event_id,
            stream_id,
            stream_label,
            event_category: event.payload.category(),
            event_key: event.payload.key(),
            description: event.payload.description(locale),
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{AppStoreData, CsbEvent, CsbMainEvent, Locale, StreamId, store::StoreEvent};

    const EN: Locale = Locale::En;

    fn stream_id() -> StreamId {
        StreamId::new()
    }

    #[test]
    fn from_main_event_sets_fields() {
        let sid = stream_id();
        let event = StoreEvent::new(1, CsbMainEvent::DeveloperLogin { stream_id: sid });

        let entry = CsbAuditLogEntry::from_main_event(event, sid, EN);

        assert_eq!(entry.event_id, 1);
        assert_eq!(entry.stream_id, sid);
        assert_eq!(entry.stream_label, "Main CSB stream");
        assert_eq!(entry.event_category, "system");
        assert_eq!(entry.event_key, "developer_login");
        assert_eq!(entry.description, "Developer login");
    }

    #[test]
    fn from_main_event_preserves_timestamp() {
        let sid = stream_id();
        let timestamp = chrono::DateTime::from_timestamp(1_700_000_000, 0).unwrap();
        let event = StoreEvent::new_at(
            2,
            CsbMainEvent::DeveloperLogin { stream_id: sid },
            timestamp,
        );

        let entry = CsbAuditLogEntry::from_main_event(event, sid, EN);

        assert_eq!(entry.created_at, timestamp);
    }

    #[test]
    fn from_event_import_sets_fields() {
        let sid = stream_id();
        let label = "Political group A".to_string();
        let event = StoreEvent::new(
            3,
            CsbEvent::Import {
                hash: [0u8; 32],
                source_stream_id: StreamId::new(),
                snapshot: Box::new(AppStoreData::default()),
            },
        );

        let entry = CsbAuditLogEntry::from_event(event, sid, label.clone(), EN);

        assert_eq!(entry.event_id, 3);
        assert_eq!(entry.stream_id, sid);
        assert_eq!(entry.stream_label, label);
        assert_eq!(entry.event_category, "import");
        assert_eq!(entry.event_key, "import");
        assert_eq!(entry.description, "Imported political group");
    }

    #[test]
    fn from_event_preserves_timestamp() {
        let sid = stream_id();
        let timestamp = chrono::DateTime::from_timestamp(1_700_000_000, 0).unwrap();
        let event = StoreEvent::new_at(6, CsbEvent::SetFinished(false), timestamp);

        let entry = CsbAuditLogEntry::from_event(event, sid, "Stream".to_string(), EN);

        assert_eq!(entry.created_at, timestamp);
    }

    #[test]
    fn translates_to_dutch() {
        let sid = stream_id();
        let event = StoreEvent::new(
            1,
            CsbEvent::Import {
                hash: [0u8; 32],
                source_stream_id: StreamId::new(),
                snapshot: Box::new(AppStoreData::default()),
            },
        );

        let entry = CsbAuditLogEntry::from_event(event, sid, "Stream".to_string(), Locale::Nl);

        assert_eq!(entry.description, "Politieke groepering geïmporteerd");
    }

    #[test]
    fn matches_search_by_description() {
        let sid = stream_id();
        let event = StoreEvent::new(1, CsbEvent::SetFinished(true));
        let entry = CsbAuditLogEntry::from_event(event, sid, "Stream".to_string(), EN);

        assert!(entry.matches_search("finished"));
        assert!(entry.matches_search("FINISHED"));
        assert!(!entry.matches_search("nonexistent"));
    }

    #[test]
    fn matches_search_by_stream_label() {
        let sid = stream_id();
        let event = StoreEvent::new(1, CsbEvent::SetFinished(true));
        let entry = CsbAuditLogEntry::from_event(event, sid, "PG Kiesraad".to_string(), EN);

        assert!(entry.matches_search("Kiesraad"));
        assert!(entry.matches_search("kiesraad"));
    }
}
