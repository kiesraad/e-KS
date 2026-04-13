use chrono::{DateTime, Utc};

use crate::{AppEvent, Locale, store::StoreEvent, trans};

pub struct AuditLogEntry {
    pub event_id: usize,
    pub description: String,
    pub details: String,
    pub created_at: DateTime<Utc>,
}

impl AuditLogEntry {
    pub fn new(event: StoreEvent<AppEvent>, locale: Locale) -> Self {
        Self {
            event_id: event.event_id,
            description: event_description(&event.payload, locale),
            details: details(&event.payload),
            created_at: event.created_at,
        }
    }
}

fn event_description(event: &AppEvent, locale: Locale) -> String {
    match event {
        AppEvent::StreamCreated { .. } => trans!("audit_log.event.stream_created", locale),
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
        AppEvent::CreateListSubmitter(ls) | AppEvent::UpdateListSubmitter(ls) => ls.name.display(),
        AppEvent::CreateSubstituteSubmitter(ss) | AppEvent::UpdateSubstituteSubmitter(ss) => {
            ss.name.display()
        }
        AppEvent::DownloadFile { file_name, .. } => file_name.clone(),
        _ => String::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        Locale, StreamId,
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
        let pg = sample_political_group();
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
        assert!(entry.details.is_empty());
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
    fn from_create_candidate_list_event_has_empty_details() {
        let list = sample_candidate_list(CandidateListId::new());
        let event = StoreEvent::new(7, AppEvent::CreateCandidateList(list));

        let entry = AuditLogEntry::new(event, EN);

        assert_eq!(entry.description, "Created list of candidates");
        assert!(entry.details.is_empty());
    }

    #[test]
    fn from_developer_login_event_has_empty_details() {
        let event = StoreEvent::new(
            8,
            AppEvent::DeveloperLogin {
                stream_id: StreamId::new(),
            },
        );

        let entry = AuditLogEntry::new(event, EN);

        assert_eq!(entry.description, "Developer login");
        assert!(entry.details.is_empty());
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
