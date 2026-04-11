use axum_extra::routing::TypedPath;
use chrono::{DateTime, Utc};

use crate::{
    AppEvent, Locale, audit_log::AuditLogDetailPath, authorised_agents::AuthorisedAgentId,
    candidate_lists::CandidateListId, list_submitters::ListSubmitterId, persons::PersonId,
    store::StoreEvent, trans,
};

pub struct AuditLogEntry {
    pub event_id: usize,
    pub description: String,
    pub details: String,
    pub subject_id: String,
    pub subject_id_full: String,
    pub subject_path: String,
    pub created_at: DateTime<Utc>,
}

impl AuditLogEntry {
    pub fn new(event: StoreEvent<AppEvent>, locale: Locale) -> Self {
        let full_id = subject_id_full(&event.payload);
        Self {
            event_id: event.event_id,
            description: event_description(&event.payload, locale),
            details: details(&event.payload),
            subject_id: abbreviate_str(&full_id),
            subject_id_full: full_id,
            subject_path: subject_path(&event.payload),
            created_at: event.created_at,
        }
    }

    pub fn detail_path(&self) -> impl TypedPath {
        AuditLogDetailPath {
            event_id: self.event_id,
        }
    }
}

fn event_description(event: &AppEvent, locale: Locale) -> String {
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
        AppEvent::UpdateCandidateListSubmitters { .. } => {
            trans!("audit_log.event.update_candidate_list_submitters", locale)
        }
        AppEvent::AddCandidateToCandidateList { .. } => {
            trans!("audit_log.event.add_candidate_to_list", locale)
        }
        AppEvent::RemoveCandidateFromCandidateList { .. } => {
            trans!("audit_log.event.remove_candidate_from_list", locale)
        }
        AppEvent::DeleteCandidateList(_) => trans!("audit_log.event.delete_candidate_list", locale),
        AppEvent::CreateAuthorisedAgent(_) => {
            trans!("audit_log.event.create_authorised_agent", locale)
        }
        AppEvent::UpdateAuthorisedAgent(_) => {
            trans!("audit_log.event.update_authorised_agent", locale)
        }
        AppEvent::DeleteAuthorisedAgent(_) => {
            trans!("audit_log.event.delete_authorised_agent", locale)
        }
        AppEvent::CreateListSubmitter(_) => trans!("audit_log.event.create_list_submitter", locale),
        AppEvent::UpdateListSubmitter(_) => trans!("audit_log.event.update_list_submitter", locale),
        AppEvent::DeleteListSubmitter { .. } => {
            trans!("audit_log.event.delete_list_submitter", locale)
        }
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
    }
}

fn details(event: &AppEvent) -> String {
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
        AppEvent::UpdatePersonAddress { person_id, .. }
        | AppEvent::UpdatePersonRepresentative { person_id, .. }
        | AppEvent::DeletePerson { person_id } => abbreviate_str(&person_id.to_string()),
        AppEvent::CreateCandidateList(cl) => district_codes(&cl.electoral_districts),
        AppEvent::UpdateCandidateListDistricts {
            electoral_districts,
            ..
        } => district_codes(electoral_districts),
        AppEvent::UpdateCandidateListOrder { list_id, .. }
        | AppEvent::UpdateCandidateListSubmitters { list_id, .. }
        | AppEvent::AddCandidateToCandidateList { list_id, .. }
        | AppEvent::RemoveCandidateFromCandidateList { list_id, .. } => {
            abbreviate_str(&list_id.to_string())
        }
        AppEvent::DeleteCandidateList(cl_id) => abbreviate_str(&cl_id.to_string()),
        AppEvent::CreateAuthorisedAgent(aa) | AppEvent::UpdateAuthorisedAgent(aa) => {
            aa.name.display()
        }
        AppEvent::DeleteAuthorisedAgent(aa_id) => abbreviate_str(&aa_id.to_string()),
        AppEvent::CreateListSubmitter(ls) | AppEvent::UpdateListSubmitter(ls) => ls.name.display(),
        AppEvent::DeleteListSubmitter {
            list_submitter_id, ..
        } => abbreviate_str(&list_submitter_id.to_string()),
        AppEvent::CreateSubstituteSubmitter(ss) | AppEvent::UpdateSubstituteSubmitter(ss) => {
            ss.name.display()
        }
        AppEvent::DeleteSubstituteSubmitter {
            substitute_submitter_id,
            ..
        } => abbreviate_str(&substitute_submitter_id.to_string()),
        AppEvent::DeveloperLogin {
            political_group_id, ..
        } => abbreviate_str(&political_group_id.to_string()),
        AppEvent::DownloadFile { file_name, .. } => file_name.clone(),
    }
}

/// Abbreviate a string to its first 8 characters.
fn abbreviate_str(s: &str) -> String {
    s[..8.min(s.len())].to_string()
}

/// Return the URL path to the subject entity's form/page, or empty if not applicable.
fn subject_path(event: &AppEvent) -> String {
    fn person_path(id: PersonId) -> String {
        format!("/persons/{id}/update")
    }
    fn candidate_list_path(id: CandidateListId) -> String {
        format!("/candidate-lists/{id}")
    }
    fn authorised_agent_path(id: AuthorisedAgentId) -> String {
        format!("/political-group/authorised-agents/{id}/update")
    }
    fn list_submitter_path(id: ListSubmitterId) -> String {
        format!("/political-group/list-submitters/{id}/update")
    }
    fn substitute_submitter_path(id: ListSubmitterId) -> String {
        format!("/political-group/substitute-submitters/{id}/update")
    }

    match event {
        AppEvent::UpdatePoliticalGroup(_) => "/political-group".to_string(),
        AppEvent::CreatePerson(p) | AppEvent::UpdatePerson(p) => person_path(p.id),
        AppEvent::CreatePersonPersonalData { person_id, .. }
        | AppEvent::UpdatePersonPersonalData { person_id, .. }
        | AppEvent::UpdatePersonAddress { person_id, .. }
        | AppEvent::UpdatePersonRepresentative { person_id, .. } => person_path(*person_id),
        AppEvent::CreateCandidateList(cl) => candidate_list_path(cl.id),
        AppEvent::UpdateCandidateListDistricts { list_id, .. }
        | AppEvent::UpdateCandidateListOrder { list_id, .. }
        | AppEvent::UpdateCandidateListSubmitters { list_id, .. }
        | AppEvent::AddCandidateToCandidateList { list_id, .. }
        | AppEvent::RemoveCandidateFromCandidateList { list_id, .. } => {
            candidate_list_path(*list_id)
        }
        AppEvent::CreateAuthorisedAgent(aa) | AppEvent::UpdateAuthorisedAgent(aa) => {
            authorised_agent_path(aa.id)
        }
        AppEvent::CreateListSubmitter(ls) | AppEvent::UpdateListSubmitter(ls) => {
            list_submitter_path(ls.id)
        }
        AppEvent::CreateSubstituteSubmitter(ss) | AppEvent::UpdateSubstituteSubmitter(ss) => {
            substitute_submitter_path(ss.id)
        }
        // Deleted entities and system events have no target page
        _ => String::new(),
    }
}

/// Extract the primary subject ID from the event as a full UUID string.
fn subject_id_full(event: &AppEvent) -> String {
    match event {
        AppEvent::UpdatePoliticalGroup(pg) => pg.id.to_string(),
        AppEvent::CreatePerson(p) | AppEvent::UpdatePerson(p) => p.id.to_string(),
        AppEvent::CreatePersonPersonalData { person_id, .. }
        | AppEvent::UpdatePersonPersonalData { person_id, .. }
        | AppEvent::UpdatePersonAddress { person_id, .. }
        | AppEvent::UpdatePersonRepresentative { person_id, .. }
        | AppEvent::DeletePerson { person_id } => person_id.to_string(),
        AppEvent::CreateCandidateList(cl) => cl.id.to_string(),
        AppEvent::UpdateCandidateListDistricts { list_id, .. }
        | AppEvent::UpdateCandidateListOrder { list_id, .. }
        | AppEvent::UpdateCandidateListSubmitters { list_id, .. }
        | AppEvent::AddCandidateToCandidateList { list_id, .. }
        | AppEvent::RemoveCandidateFromCandidateList { list_id, .. } => list_id.to_string(),
        AppEvent::DeleteCandidateList(cl_id) => cl_id.to_string(),
        AppEvent::CreateAuthorisedAgent(aa) | AppEvent::UpdateAuthorisedAgent(aa) => {
            aa.id.to_string()
        }
        AppEvent::DeleteAuthorisedAgent(aa_id) => aa_id.to_string(),
        AppEvent::CreateListSubmitter(ls) | AppEvent::UpdateListSubmitter(ls) => ls.id.to_string(),
        AppEvent::DeleteListSubmitter {
            list_submitter_id, ..
        } => list_submitter_id.to_string(),
        AppEvent::CreateSubstituteSubmitter(ss) | AppEvent::UpdateSubstituteSubmitter(ss) => {
            ss.id.to_string()
        }
        AppEvent::DeleteSubstituteSubmitter {
            substitute_submitter_id,
            ..
        } => substitute_submitter_id.to_string(),
        AppEvent::DeveloperLogin {
            political_group_id, ..
        } => political_group_id.to_string(),
        AppEvent::DownloadFile { list_id, .. } => list_id.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        Locale, PoliticalGroupId,
        authorised_agents::AuthorisedAgentId,
        candidate_lists::CandidateListId,
        list_submitters::ListSubmitterId,
        persons::PersonId,
        test_utils::{
            sample_authorised_agent, sample_candidate_list, sample_list_submitter, sample_person,
            sample_political_group,
        },
    };

    const EN: Locale = Locale::En;

    #[test]
    fn from_create_person_event() {
        let person = sample_person(PersonId::new());
        let expected_name = person.name.display();
        let event = StoreEvent::new(1, AppEvent::CreatePerson(person));

        let entry = AuditLogEntry::new(event, EN);

        assert_eq!(entry.event_id, 1);
        assert_eq!(entry.description, "Created person");
        assert_eq!(entry.details, expected_name);
    }

    #[test]
    fn from_update_political_group_event() {
        let pg = sample_political_group(PoliticalGroupId::new());
        let expected_name = pg.display_name.as_ref().unwrap().to_string();
        let event = StoreEvent::new(2, AppEvent::UpdatePoliticalGroup(pg));

        let entry = AuditLogEntry::new(event, EN);

        assert_eq!(entry.event_id, 2);
        assert_eq!(entry.description, "Updated political group");
        assert_eq!(entry.details, expected_name);
    }

    #[test]
    fn from_delete_person_event_has_empty_details() {
        let person_id = PersonId::new();
        let event = StoreEvent::new(3, AppEvent::DeletePerson { person_id });

        let entry = AuditLogEntry::new(event, EN);

        assert_eq!(entry.description, "Deleted person");
        assert!(!entry.details.is_empty());
    }

    #[test]
    fn from_create_authorised_agent_event() {
        let agent = sample_authorised_agent(AuthorisedAgentId::new());
        let expected_name = agent.name.display();
        let event = StoreEvent::new(4, AppEvent::CreateAuthorisedAgent(agent));

        let entry = AuditLogEntry::new(event, EN);

        assert_eq!(entry.description, "Created authorised agent");
        assert_eq!(entry.details, expected_name);
    }

    #[test]
    fn from_create_list_submitter_event() {
        let submitter = sample_list_submitter(ListSubmitterId::new());
        let expected_name = submitter.name.display();
        let event = StoreEvent::new(5, AppEvent::CreateListSubmitter(submitter));

        let entry = AuditLogEntry::new(event, EN);

        assert_eq!(entry.description, "Created submitter of the list");
        assert_eq!(entry.details, expected_name);
    }

    #[test]
    fn from_download_file_event() {
        let event = StoreEvent::new(
            6,
            AppEvent::DownloadFile {
                file_name: "model-h1.pdf".to_string(),
                download_path: "/download/h1".to_string(),
                list_id: CandidateListId::new(),
            },
        );

        let entry = AuditLogEntry::new(event, EN);

        assert_eq!(entry.description, "Downloaded file");
        assert_eq!(entry.details, "model-h1.pdf");
    }

    #[test]
    fn from_create_candidate_list_event_shows_districts() {
        let list = sample_candidate_list(CandidateListId::new());
        let event = StoreEvent::new(7, AppEvent::CreateCandidateList(list));

        let entry = AuditLogEntry::new(event, EN);

        assert_eq!(entry.description, "Created list of candidates");
        assert_eq!(entry.details, "UT");
    }

    #[test]
    fn from_developer_login_event_shows_abbreviated_id() {
        let pg_id = PoliticalGroupId::new();
        let event = StoreEvent::new(
            8,
            AppEvent::DeveloperLogin {
                political_group_id: pg_id,
            },
        );

        let entry = AuditLogEntry::new(event, EN);

        assert_eq!(entry.description, "Developer login");
        assert_eq!(entry.details, abbreviate_str(&pg_id.to_string()));
    }

    #[test]
    fn translates_to_dutch() {
        let person_id = PersonId::new();
        let event = StoreEvent::new(1, AppEvent::DeletePerson { person_id });

        let entry = AuditLogEntry::new(event, Locale::Nl);

        assert_eq!(entry.description, "Persoon verwijderd");
    }

    #[test]
    fn preserves_event_timestamp() {
        let timestamp = chrono::DateTime::from_timestamp(1_700_000_000, 0).unwrap();
        let event = StoreEvent::new_at(
            1,
            AppEvent::DeletePerson {
                person_id: PersonId::new(),
            },
            timestamp,
        );

        let entry = AuditLogEntry::new(event, EN);

        assert_eq!(entry.created_at, timestamp);
    }
}
