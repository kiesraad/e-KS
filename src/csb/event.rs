use serde::{Deserialize, Serialize};

use crate::{AppStoreData, StreamId};

/// Domain events that mutate the CSB (Centraal Stembureau) store.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum CsbEvent {
    /// Import a submitted candidate-list package, identified by the chain hash
    /// of the event stream it was produced from.
    ///
    /// Carries a snapshot of the source [`AppStoreData`] reconstructed by
    /// replaying the source stream up to the matched event (see
    /// [`AppStoreData::snapshot_until`]). The import is persisted under a fresh
    /// CSB stream (never the source partition, which holds the app's own
    /// events), so `source_stream_id` is recorded for reference. The election is
    /// not: it is copied onto the CSB stream's own `(stream_id, election)` key.
    Import {
        /// Chain hash of the package, as entered by the committee.
        hash: String,
        /// Stream the imported package was produced from.
        source_stream_id: StreamId,
        /// Snapshot of the source projection at the matched event, with its own
        /// event log excluded. Boxed to keep the event enum small.
        snapshot: Box<AppStoreData>,
    },
    ToggleFinish,
}

impl CsbEvent {
    /// Return a stable category key for filtering in the audit log.
    pub fn event_category(&self) -> &'static str {
        match self {
            CsbEvent::Import { .. } => "import",
            CsbEvent::ToggleFinish => "toggle_finish",
        }
    }

    /// Return a stable snake_case key identifying the event variant.
    pub fn event_key(&self) -> &'static str {
        match self {
            CsbEvent::Import { .. } => "import",
            CsbEvent::ToggleFinish => "toggle_finish",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn import_event() -> CsbEvent {
        CsbEvent::Import {
            hash: "abc123".to_string(),
            source_stream_id: StreamId::default(),
            snapshot: Box::new(AppStoreData::default()),
        }
    }

    #[test]
    fn import_event_category() {
        assert_eq!(import_event().event_category(), "import");
    }

    #[test]
    fn import_event_key() {
        assert_eq!(import_event().event_key(), "import");
    }
}
