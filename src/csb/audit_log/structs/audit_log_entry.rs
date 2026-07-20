use axum_extra::routing::TypedPath;
use chrono::{DateTime, Utc};

use crate::{
    Event, Locale, QueryParamState, StreamId, csb::audit_log::pages::CsbAuditLogDetailPath,
    store::StoreEvent,
};

/// A single row in the CSB audit log, covering events from both the global
/// main stream and per-import political-group streams.
pub struct CsbAuditLogEntry {
    pub event_id: usize,
    pub stream_id: StreamId,
    /// Human-readable label for the source stream
    pub stream_label: String,
    pub description: String,
    pub created_at: DateTime<Utc>,
}

impl CsbAuditLogEntry {
    /// Build a row from any stored event and a pre-computed stream label.
    pub fn from_event<E: Event>(
        event: &StoreEvent<E>,
        stream_id: StreamId,
        stream_label: String,
        locale: Locale,
    ) -> Self {
        Self {
            event_id: event.event_id,
            stream_id,
            stream_label,
            description: event.payload.description(locale),
            created_at: event.created_at,
        }
    }

    pub fn matches_search(&self, query: &str) -> bool {
        let query = query.to_lowercase();
        self.description.to_lowercase().contains(&query)
            || self.stream_label.to_lowercase().contains(&query)
    }

    /// Detail path carrying `return_to` as a `redirect_to` query param, so
    /// closing the detail overlay returns to the same filtered list page.
    pub fn detail_path_with_return(&self, return_to: &str) -> String {
        CsbAuditLogDetailPath {
            stream_id: self.stream_id,
            event_id: self.event_id,
        }
        .with_query_params(QueryParamState::redirect_to(return_to.to_string()))
        .to_string()
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

        let entry = CsbAuditLogEntry::from_event(&event, sid, "Main CSB stream".to_string(), EN);

        assert_eq!(entry.event_id, 1);
        assert_eq!(entry.stream_id, sid);
        assert_eq!(entry.stream_label, "Main CSB stream");
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

        let entry = CsbAuditLogEntry::from_event(&event, sid, "Main CSB stream".to_string(), EN);

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

        let entry = CsbAuditLogEntry::from_event(&event, sid, label.clone(), EN);

        assert_eq!(entry.event_id, 3);
        assert_eq!(entry.stream_id, sid);
        assert_eq!(entry.stream_label, label);
        assert_eq!(entry.description, "Imported political group");
    }

    #[test]
    fn from_event_preserves_timestamp() {
        let sid = stream_id();
        let timestamp = chrono::DateTime::from_timestamp(1_700_000_000, 0).unwrap();
        let event = StoreEvent::new_at(6, CsbEvent::SetFinished(false), timestamp);

        let entry = CsbAuditLogEntry::from_event(&event, sid, "Stream".to_string(), EN);

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

        let entry = CsbAuditLogEntry::from_event(&event, sid, "Stream".to_string(), Locale::Nl);

        assert_eq!(entry.description, "Politieke groepering geïmporteerd");
    }

    #[test]
    fn matches_search_by_description() {
        let sid = stream_id();
        let event = StoreEvent::new(1, CsbEvent::SetFinished(true));
        let entry = CsbAuditLogEntry::from_event(&event, sid, "Stream".to_string(), EN);

        assert!(entry.matches_search("finished"));
        assert!(entry.matches_search("FINISHED"));
        assert!(!entry.matches_search("nonexistent"));
    }

    #[test]
    fn matches_search_by_stream_label() {
        let sid = stream_id();
        let event = StoreEvent::new(1, CsbEvent::SetFinished(true));
        let entry = CsbAuditLogEntry::from_event(&event, sid, "PG Kiesraad".to_string(), EN);

        assert!(entry.matches_search("Kiesraad"));
        assert!(entry.matches_search("kiesraad"));
    }

    #[test]
    fn event_category_and_key_are_set_correctly() {
        let sid = stream_id();
        let event = CsbMainEvent::DeveloperLogin { stream_id: sid };
        assert_eq!(event.category(), "system");
        assert_eq!(event.key(), "developer_login");

        let event = CsbEvent::Import {
            hash: [0u8; 32],
            source_stream_id: StreamId::new(),
            snapshot: Box::new(AppStoreData::default()),
        };
        assert_eq!(event.category(), "import");
        assert_eq!(event.key(), "import");
    }
}
