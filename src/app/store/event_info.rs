//! Event-level helpers shared by `AuditLogEntry` and `AuditLogDetail`.
//!
//! Each function maps an `AppEvent` to a single piece of information needed
//! by the audit log UI (translated description, human-readable details,
//! subject entity URL, and the primary subject ID).

use crate::{
    AppEvent, Locale,
    candidate_lists::{CandidateListId, ViewCandidateListPath},
    list_submitters::ListSubmitter,
    persons::{PersonId, UpdatePersonPath},
    political_groups::PoliticalGroupUpdatePath,
    trans,
    utils::format_hash,
};

/// Translated label describing what the event did.
pub(super) fn event_description(event: &AppEvent, locale: Locale) -> String {
    match event {
        AppEvent::UpdatePoliticalGroup(_) => {
            trans!("audit_log.event.update_political_group", locale)
        }
        AppEvent::CreatePerson(_) | AppEvent::CreatePersonPersonalData { .. } => {
            trans!("audit_log.event.create_person", locale)
        }
        AppEvent::UpdatePerson(_) | AppEvent::UpdatePersonPersonalData { .. } => {
            trans!("audit_log.event.update_person", locale)
        }
        AppEvent::UpdatePersonAddress { .. } => {
            trans!("audit_log.event.update_person_address", locale)
        }
        AppEvent::UpdatePersonRepresentative { .. } => {
            trans!("audit_log.event.update_person_representative", locale)
        }
        AppEvent::DeletePerson { .. } => trans!("audit_log.event.delete_person", locale),
        AppEvent::CreateCandidateList(_) => trans!("audit_log.event.create_candidate_list", locale),
        AppEvent::UpdateCandidateListDistricts { .. } => {
            trans!("audit_log.event.update_candidate_list_districts", locale)
        }
        AppEvent::UpdateCandidateListOrder { .. } => {
            trans!("audit_log.event.update_candidate_list_order", locale)
        }
        AppEvent::AddCandidateToCandidateList { .. } => {
            trans!("audit_log.event.add_candidate_to_list", locale)
        }
        AppEvent::RemoveCandidateFromCandidateList { .. } => {
            trans!("audit_log.event.remove_candidate_from_list", locale)
        }
        AppEvent::DeleteCandidateList(_) => trans!("audit_log.event.delete_candidate_list", locale),
        AppEvent::CreateNameAuthorisation(_) => {
            trans!("audit_log.event.create_name_authorisation", locale)
        }
        AppEvent::UpdateNameAuthorisation(_) => {
            trans!("audit_log.event.update_name_authorisation", locale)
        }
        AppEvent::DeleteNameAuthorisation(_) => {
            trans!("audit_log.event.delete_name_authorisation", locale)
        }
        AppEvent::UpdateListSubmitter(_) => trans!("audit_log.event.update_list_submitter", locale),
        AppEvent::CreateSubstituteSubmitter(_) => {
            trans!("audit_log.event.create_substitute_submitter", locale)
        }
        AppEvent::UpdateSubstituteSubmitter(_) => {
            trans!("audit_log.event.update_substitute_submitter", locale)
        }
        AppEvent::DeleteSubstituteSubmitter { .. } => {
            trans!("audit_log.event.delete_substitute_submitter", locale)
        }
        AppEvent::DeveloperLogin { .. } => trans!("audit_log.event.developer_login", locale),
        AppEvent::DownloadFile { .. } => trans!("audit_log.event.download_file", locale),
        AppEvent::HideDownloadWarning => trans!("audit_log.event.hide_download_warning", locale),
        AppEvent::ExportCsv { .. } => trans!("audit_log.event.export_csv", locale),
        AppEvent::ImportCandidates { .. } => {
            trans!("audit_log.event.import_csv", locale)
        }
        AppEvent::Import { .. } => trans!("audit_log.event.import", locale),
    }
}

/// Short human-readable details for a listing row (name, file, districts, ...).
pub(super) fn event_details(event: &AppEvent) -> String {
    fn district_codes(districts: &[crate::ElectoralDistrict]) -> String {
        districts
            .iter()
            .map(|d| d.code())
            .collect::<Vec<_>>()
            .join(", ")
    }

    match event {
        AppEvent::UpdatePoliticalGroup(pg) => pg
            .display_name
            .as_ref()
            .map(|n| n.to_string())
            .unwrap_or_default(),
        AppEvent::CreatePerson(p) | AppEvent::UpdatePerson(p) => p.name.display(),
        AppEvent::CreatePersonPersonalData { name, .. }
        | AppEvent::UpdatePersonPersonalData { name, .. } => name.display(),
        AppEvent::CreateCandidateList(cl) => district_codes(&cl.electoral_districts),
        AppEvent::UpdateCandidateListDistricts {
            electoral_districts,
            ..
        } => district_codes(electoral_districts),
        AppEvent::CreateNameAuthorisation(aa) | AppEvent::UpdateNameAuthorisation(aa) => {
            format!("{} ({})", aa.legal_name, aa.name.display())
        }
        AppEvent::UpdateListSubmitter(ls) => ls.name.display(),
        AppEvent::CreateSubstituteSubmitter(ss) | AppEvent::UpdateSubstituteSubmitter(ss) => {
            ss.name.display()
        }
        AppEvent::DownloadFile { file_name, .. }
        | AppEvent::ExportCsv { file_name, .. }
        | AppEvent::ImportCandidates { file_name, .. } => file_name.clone(),
        AppEvent::Import { hash } => format_hash(hash, true),
        AppEvent::UpdatePersonAddress { .. }
        | AppEvent::UpdatePersonRepresentative { .. }
        | AppEvent::DeletePerson { .. }
        | AppEvent::UpdateCandidateListOrder { .. }
        | AppEvent::AddCandidateToCandidateList { .. }
        | AppEvent::RemoveCandidateFromCandidateList { .. }
        | AppEvent::DeleteCandidateList(..)
        | AppEvent::DeleteNameAuthorisation(..)
        | AppEvent::DeleteSubstituteSubmitter { .. }
        | AppEvent::DeveloperLogin { .. }
        | AppEvent::HideDownloadWarning => AppEvent::DEFAULT_DETAILS.to_string(),
    }
}

impl AppEvent {
    pub const DEFAULT_DETAILS: &str = "-";

    /// Primary subject ID (full UUID string); empty for events without one.
    pub fn subject_id_full(&self) -> String {
        match self {
            AppEvent::UpdatePoliticalGroup(_) => String::new(),
            AppEvent::CreatePerson(p) | AppEvent::UpdatePerson(p) => p.id.to_string(),
            AppEvent::CreatePersonPersonalData { person_id, .. }
            | AppEvent::UpdatePersonPersonalData { person_id, .. }
            | AppEvent::UpdatePersonAddress { person_id, .. }
            | AppEvent::UpdatePersonRepresentative { person_id, .. }
            | AppEvent::DeletePerson { person_id } => person_id.to_string(),
            AppEvent::CreateCandidateList(cl) => cl.id.to_string(),
            AppEvent::UpdateCandidateListDistricts { list_id, .. }
            | AppEvent::UpdateCandidateListOrder { list_id, .. }
            | AppEvent::AddCandidateToCandidateList { list_id, .. }
            | AppEvent::RemoveCandidateFromCandidateList { list_id, .. } => list_id.to_string(),
            AppEvent::DeleteCandidateList(cl_id) => cl_id.to_string(),
            AppEvent::CreateNameAuthorisation(aa) | AppEvent::UpdateNameAuthorisation(aa) => {
                aa.id.to_string()
            }
            AppEvent::DeleteNameAuthorisation(aa_id) => aa_id.to_string(),
            AppEvent::UpdateListSubmitter(_) => String::new(),
            AppEvent::CreateSubstituteSubmitter(ss) | AppEvent::UpdateSubstituteSubmitter(ss) => {
                ss.id.to_string()
            }
            AppEvent::DeleteSubstituteSubmitter {
                substitute_submitter_id,
                ..
            } => substitute_submitter_id.to_string(),
            AppEvent::DeveloperLogin { stream_id, .. } => stream_id.to_string(),
            AppEvent::ExportCsv { list_id, .. } | AppEvent::ImportCandidates { list_id, .. } => {
                list_id.to_string()
            }
            AppEvent::DownloadFile { .. }
            | AppEvent::HideDownloadWarning
            | AppEvent::Import { .. } => String::new(),
        }
    }

    /// URL to the subject entity's page, or empty when none applies (deleted
    /// entities, system events).
    pub fn subject_path(&self) -> String {
        fn person_path(person_id: PersonId) -> String {
            UpdatePersonPath { person_id }.to_string()
        }
        fn candidate_list_path(list_id: CandidateListId) -> String {
            ViewCandidateListPath { list_id }.to_string()
        }

        match self {
            AppEvent::UpdatePoliticalGroup(_) => PoliticalGroupUpdatePath {}.to_string(),
            AppEvent::CreatePerson(p) | AppEvent::UpdatePerson(p) => person_path(p.id),
            AppEvent::CreatePersonPersonalData { person_id, .. }
            | AppEvent::UpdatePersonPersonalData { person_id, .. }
            | AppEvent::UpdatePersonAddress { person_id, .. }
            | AppEvent::UpdatePersonRepresentative { person_id, .. } => person_path(*person_id),
            AppEvent::CreateCandidateList(cl) => cl.view_path().to_string(),
            AppEvent::UpdateCandidateListDistricts { list_id, .. }
            | AppEvent::UpdateCandidateListOrder { list_id, .. }
            | AppEvent::AddCandidateToCandidateList { list_id, .. }
            | AppEvent::RemoveCandidateFromCandidateList { list_id, .. } => {
                candidate_list_path(*list_id)
            }
            AppEvent::CreateNameAuthorisation(aa) | AppEvent::UpdateNameAuthorisation(aa) => {
                aa.update_path().to_string()
            }
            AppEvent::UpdateListSubmitter(_) => ListSubmitter::update_path().to_string(),
            AppEvent::CreateSubstituteSubmitter(ss) | AppEvent::UpdateSubstituteSubmitter(ss) => {
                ss.substitute_update_path().to_string()
            }
            AppEvent::ExportCsv { list_id, .. } | AppEvent::ImportCandidates { list_id, .. } => {
                candidate_list_path(*list_id)
            }
            _ => String::new(),
        }
    }
}
