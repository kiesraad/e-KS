use chrono::{DateTime, Utc};

use crate::{AppEvent, store::StoreEvent};

pub struct AuditLogEntry {
    pub event_id: usize,
    pub description_key: &'static str,
    pub details: String,
    pub created_at: DateTime<Utc>,
}

impl From<StoreEvent<AppEvent>> for AuditLogEntry {
    fn from(event: StoreEvent<AppEvent>) -> Self {
        Self {
            event_id: event.event_id,
            description_key: translation_key(&event.payload),
            details: details(&event.payload),
            created_at: event.created_at,
        }
    }
}

fn translation_key(event: &AppEvent) -> &'static str {
    match event {
        AppEvent::UpdatePoliticalGroup(_) => "audit_log.event.update_political_group",
        AppEvent::CreatePerson(_) | AppEvent::CreatePersonPersonalData { .. } => {
            "audit_log.event.create_person"
        }
        AppEvent::UpdatePerson(_) | AppEvent::UpdatePersonPersonalData { .. } => {
            "audit_log.event.update_person"
        }
        AppEvent::UpdatePersonAddress { .. } => "audit_log.event.update_person_address",
        AppEvent::UpdatePersonRepresentative { .. } => {
            "audit_log.event.update_person_representative"
        }
        AppEvent::DeletePerson { .. } => "audit_log.event.delete_person",
        AppEvent::CreateCandidateList(_) => "audit_log.event.create_candidate_list",
        AppEvent::UpdateCandidateListDistricts { .. } => {
            "audit_log.event.update_candidate_list_districts"
        }
        AppEvent::UpdateCandidateListOrder { .. } => {
            "audit_log.event.update_candidate_list_order"
        }
        AppEvent::UpdateCandidateListSubmitters { .. } => {
            "audit_log.event.update_candidate_list_submitters"
        }
        AppEvent::AddCandidateToCandidateList { .. } => {
            "audit_log.event.add_candidate_to_list"
        }
        AppEvent::RemoveCandidateFromCandidateList { .. } => {
            "audit_log.event.remove_candidate_from_list"
        }
        AppEvent::DeleteCandidateList(_) => "audit_log.event.delete_candidate_list",
        AppEvent::CreateAuthorisedAgent(_) => "audit_log.event.create_authorised_agent",
        AppEvent::UpdateAuthorisedAgent(_) => "audit_log.event.update_authorised_agent",
        AppEvent::DeleteAuthorisedAgent(_) => "audit_log.event.delete_authorised_agent",
        AppEvent::CreateListSubmitter(_) => "audit_log.event.create_list_submitter",
        AppEvent::UpdateListSubmitter(_) => "audit_log.event.update_list_submitter",
        AppEvent::DeleteListSubmitter { .. } => "audit_log.event.delete_list_submitter",
        AppEvent::CreateSubstituteSubmitter(_) => {
            "audit_log.event.create_substitute_submitter"
        }
        AppEvent::UpdateSubstituteSubmitter(_) => {
            "audit_log.event.update_substitute_submitter"
        }
        AppEvent::DeleteSubstituteSubmitter { .. } => {
            "audit_log.event.delete_substitute_submitter"
        }
        AppEvent::DeveloperLogin { .. } => "audit_log.event.developer_login",
        AppEvent::DownloadFile { .. } => "audit_log.event.download_file",
    }
}

fn details(event: &AppEvent) -> String {
    match event {
        AppEvent::UpdatePoliticalGroup(pg) => pg
            .display_name
            .as_ref()
            .map(|n| n.to_string())
            .unwrap_or_default(),
        AppEvent::CreatePerson(p) | AppEvent::UpdatePerson(p) => p.name.display(),
        AppEvent::CreatePersonPersonalData { name, .. }
        | AppEvent::UpdatePersonPersonalData { name, .. } => name.display(),
        AppEvent::CreateAuthorisedAgent(aa) | AppEvent::UpdateAuthorisedAgent(aa) => {
            aa.name.display()
        }
        AppEvent::CreateListSubmitter(ls) | AppEvent::UpdateListSubmitter(ls) => {
            ls.name.display()
        }
        AppEvent::CreateSubstituteSubmitter(ss) | AppEvent::UpdateSubstituteSubmitter(ss) => {
            ss.name.display()
        }
        AppEvent::DownloadFile { file_name, .. } => file_name.clone(),
        _ => String::new(),
    }
}

#[cfg(test)]
mod tests {
    /// Verify that every translation key returned by `translation_key()`
    /// is present in the locale files.
    ///
    /// The keys below are referenced via `trans!` so that the `find_used_keys`
    /// build-time check can discover them.
    #[test]
    fn translation_keys_are_registered() {
        use crate::trans;
        let en = crate::Locale::En;
        assert_ne!(trans!("audit_log.event.update_political_group", en), "");
        assert_ne!(trans!("audit_log.event.create_person", en), "");
        assert_ne!(trans!("audit_log.event.update_person", en), "");
        assert_ne!(trans!("audit_log.event.update_person_address", en), "");
        assert_ne!(trans!("audit_log.event.update_person_representative", en), "");
        assert_ne!(trans!("audit_log.event.delete_person", en), "");
        assert_ne!(trans!("audit_log.event.create_candidate_list", en), "");
        assert_ne!(trans!("audit_log.event.update_candidate_list_districts", en), "");
        assert_ne!(trans!("audit_log.event.update_candidate_list_order", en), "");
        assert_ne!(trans!("audit_log.event.update_candidate_list_submitters", en), "");
        assert_ne!(trans!("audit_log.event.add_candidate_to_list", en), "");
        assert_ne!(trans!("audit_log.event.remove_candidate_from_list", en), "");
        assert_ne!(trans!("audit_log.event.delete_candidate_list", en), "");
        assert_ne!(trans!("audit_log.event.create_authorised_agent", en), "");
        assert_ne!(trans!("audit_log.event.update_authorised_agent", en), "");
        assert_ne!(trans!("audit_log.event.delete_authorised_agent", en), "");
        assert_ne!(trans!("audit_log.event.create_list_submitter", en), "");
        assert_ne!(trans!("audit_log.event.update_list_submitter", en), "");
        assert_ne!(trans!("audit_log.event.delete_list_submitter", en), "");
        assert_ne!(trans!("audit_log.event.create_substitute_submitter", en), "");
        assert_ne!(trans!("audit_log.event.update_substitute_submitter", en), "");
        assert_ne!(trans!("audit_log.event.delete_substitute_submitter", en), "");
        assert_ne!(trans!("audit_log.event.developer_login", en), "");
        assert_ne!(trans!("audit_log.event.download_file", en), "");
    }
}
