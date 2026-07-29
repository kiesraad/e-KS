use serde::{Deserialize, Serialize};

use crate::{
    Event, PgEvent, PgStoreData, StreamId,
    structs::csb::{Correction, Omission, OmissionId},
    trans,
    utils::format_hash,
};

/// Domain events that mutate the CSB (Centraal Stembureau) store.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum CsbEvent {
    /// Import a submitted candidate-list package, identified by the chain hash
    /// of the event stream it was produced from.
    ///
    /// Carries a snapshot of the source [`PgStoreData`] reconstructed by
    /// replaying the source stream up to the matched event (see
    /// [`PgStoreData::snapshot_until`]). The import is persisted under a fresh
    /// CSB stream (never the source partition, which holds the PG stream's own
    /// events), so `source_stream_id` is recorded for reference. The election is
    /// not: it is copied onto the CSB stream's own `(stream_id, election)` key.
    Import {
        /// Hash of the imported event
        hash: [u8; 32],
        /// Stream the imported package was produced from
        source_stream_id: StreamId,
        /// Snapshot of the source projection at the matched event, with its own
        /// event log excluded. Boxed to keep the event enum small.
        snapshot: Box<PgStoreData>,
    },
    /// Create an empty political-group store without importing from a PG stream.
    CreateEmpty,
    /// An app event applied to the paper-corrected projection instead of a
    /// political group's own stream. Boxed to keep the event enum small.
    PaperCorrectedUpdate(Box<PgEvent>),
    SetFinished(bool),
    CreateOmission(Omission),
    UpdateOmission(Omission),
    DeleteOmission {
        omission_id: OmissionId,
    },
    UpdateCorrection(Correction),
}

impl Event for CsbEvent {
    fn category(&self) -> &'static str {
        match self {
            CsbEvent::Import { .. } => "import",
            CsbEvent::CreateEmpty => "import",
            CsbEvent::PaperCorrectedUpdate(_) => "paper_correction",
            CsbEvent::SetFinished(_) => "set_finished",
            CsbEvent::CreateOmission(_)
            | CsbEvent::UpdateOmission(_)
            | CsbEvent::DeleteOmission { .. } => "omission",
            CsbEvent::UpdateCorrection(_) => "correction",
        }
    }

    fn key(&self) -> &'static str {
        match self {
            CsbEvent::Import { .. } => "import",
            CsbEvent::CreateEmpty => "create_empty",
            CsbEvent::PaperCorrectedUpdate(event) => event.key(),
            CsbEvent::SetFinished(_) => "set_finished",
            CsbEvent::CreateOmission(_) => "create_omission",
            CsbEvent::UpdateOmission(_) => "update_omission",
            CsbEvent::DeleteOmission { .. } => "delete_omission",
            CsbEvent::UpdateCorrection(_) => "update_correction",
        }
    }

    fn description(&self, locale: crate::Locale) -> String {
        match self {
            CsbEvent::Import { .. } => trans!("audit_log.event.import", locale),
            CsbEvent::CreateEmpty => trans!("audit_log.event.create_empty", locale),
            CsbEvent::PaperCorrectedUpdate(event) => event.description(locale),
            CsbEvent::SetFinished(_) => trans!("audit_log.event.set_finished", locale),
            CsbEvent::CreateOmission(_) => trans!("audit_log.event.create_omission", locale),
            CsbEvent::UpdateOmission(_) => trans!("audit_log.event.update_omission", locale),
            CsbEvent::DeleteOmission { .. } => trans!("audit_log.event.delete_omission", locale),
            CsbEvent::UpdateCorrection { .. } => {
                trans!("audit_log.event.update_correction", locale)
            }
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
            CsbEvent::CreateEmpty => String::new(),
            CsbEvent::PaperCorrectedUpdate(event) => event.details(),
            CsbEvent::SetFinished(value) => value.to_string(),
            CsbEvent::CreateOmission(o) | CsbEvent::UpdateOmission(o) => o.description.clone(),
            CsbEvent::DeleteOmission { omission_id } => omission_id.to_string(),
            CsbEvent::UpdateCorrection(_) => String::new(),
        }
    }

    fn changes(&self, locale: crate::Locale) -> Vec<crate::structs::audit_log::FieldChange> {
        match self {
            CsbEvent::UpdateCorrection(correction) => vec![correction.change(locale)],
            _ => vec![],
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
            snapshot: Box::new(PgStoreData::default()),
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

    /// The audit-log metadata of a paper correction delegates to the wrapped
    /// app event, under its own category.
    #[test]
    fn paper_corrected_update_delegates_to_inner_event() {
        let event = CsbEvent::PaperCorrectedUpdate(Box::new(PgEvent::UpdatePoliticalGroup(
            crate::structs::political_groups::PoliticalGroup::default(),
        )));

        assert_eq!(event.category(), "paper_correction");
        assert_eq!(event.key(), "update_political_group");
        assert_eq!(
            event.description(crate::Locale::En),
            "Updated political group"
        );
    }
}
