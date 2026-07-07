use serde::{Deserialize, Serialize};

use crate::{
    AppStoreData, Event, StreamId,
    csb::{Omission, OmissionId},
    trans,
    utils::format_hash,
};

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
        /// Hash of the imported event
        hash: [u8; 32],
        /// Stream the imported package was produced from
        source_stream_id: StreamId,
        /// Snapshot of the source projection at the matched event, with its own
        /// event log excluded. Boxed to keep the event enum small.
        snapshot: Box<AppStoreData>,
    },
    SetFinished(bool),
    CreateOmission(Omission),
    UpdateOmission(Omission),
    DeleteOmission {
        omission_id: OmissionId,
    },
}

impl Event for CsbEvent {
    fn category(&self) -> &'static str {
        match self {
            CsbEvent::Import { .. } => "import",
            CsbEvent::SetFinished(_) => "set_finished",
            CsbEvent::CreateOmission(_)
            | CsbEvent::UpdateOmission(_)
            | CsbEvent::DeleteOmission { .. } => "omission",
        }
    }

    fn key(&self) -> &'static str {
        match self {
            CsbEvent::Import { .. } => "import",
            CsbEvent::SetFinished(_) => "set_finished",
            CsbEvent::CreateOmission(_) => "create_omission",
            CsbEvent::UpdateOmission(_) => "update_omission",
            CsbEvent::DeleteOmission { .. } => "delete_omission",
        }
    }

    fn description(&self, locale: crate::Locale) -> String {
        match self {
            CsbEvent::Import { .. } => trans!("audit_log.event.import", locale),
            CsbEvent::SetFinished(_) => trans!("audit_log.event.set_finished", locale),
            CsbEvent::CreateOmission(_) => trans!("audit_log.event.create_omission", locale),
            CsbEvent::UpdateOmission(_) => trans!("audit_log.event.update_omission", locale),
            CsbEvent::DeleteOmission { .. } => trans!("audit_log.event.delete_omission", locale),
        }
    }

    fn details(&self) -> String {
        match self {
            CsbEvent::Import {
                hash,
                source_stream_id,
                ..
            } => {
                format!(
                    "Hash: {}\nSource stream: {source_stream_id}",
                    format_hash(hash, true)
                )
            }
            CsbEvent::SetFinished(value) => value.to_string(),
            CsbEvent::CreateOmission(o) | CsbEvent::UpdateOmission(o) => o.description.clone(),
            CsbEvent::DeleteOmission { omission_id } => omission_id.to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn import_event() -> CsbEvent {
        CsbEvent::Import {
            hash: [42; 32],
            source_stream_id: StreamId::default(),
            snapshot: Box::new(AppStoreData::default()),
        }
    }

    #[test]
    fn import_event_category() {
        assert_eq!(import_event().category(), "import");
    }

    #[test]
    fn import_event_key() {
        assert_eq!(import_event().key(), "import");
    }
}
