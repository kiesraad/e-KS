//! Extract the old/new JSON snapshots for a given event, used by
//! `AuditLogDetail::compute` to build a field-level diff.
//!
//! - Create events: no old state; new contains the created entity.
//! - Update events: old is the prior entity from `state_before`; new is from
//!   `state_after`.
//! - Delete events: old is the entity from `state_before`; no new state.
//! - System events: informational only; rendered as additions.

use crate::{AppEvent, AppStoreData};

pub(super) fn extract_old_new(
    event: &AppEvent,
    state_before: &AppStoreData,
    state_after: &AppStoreData,
) -> (Option<serde_json::Value>, Option<serde_json::Value>) {
    macro_rules! update {
        ($kind:ident) => {{
            let old = serde_json::to_value(&state_before.$kind).ok();
            let new = serde_json::to_value(&state_after.$kind).ok();
            (old, new)
        }};

        ($kind:ident, $id:expr) => {{
            let old = state_before
                .$kind
                .get(&$id)
                .and_then(|data| serde_json::to_value(data).ok());
            let new = state_after
                .$kind
                .get(&$id)
                .and_then(|data| serde_json::to_value(data).ok());
            (old, new)
        }};
    }

    macro_rules! event {
        ({
            $($field:ident: $val:expr),* $(,)?
        }) => {{
            let val = serde_json::json!({ $(stringify!($field): $val,)* });
            (None, Some(val))
        }};
    }

    match event {
        AppEvent::CreatePerson(person) => update!(persons, person.id),
        // Technically a create, but update works too as it does a diff
        AppEvent::CreatePersonPersonalData { person_id, .. } => update!(persons, person_id),
        AppEvent::CreateCandidateList(cl) => update!(candidate_lists, cl.id),
        AppEvent::CreateAuthorisedAgent(aa) => update!(authorised_agents, aa.id),
        AppEvent::CreateSubstituteSubmitter(ss) => update!(substitute_submitters, ss.id),

        AppEvent::UpdatePerson(person) => update!(persons, person.id),
        AppEvent::UpdateAuthorisedAgent(aa) => update!(authorised_agents, aa.id),
        AppEvent::UpdateListSubmitter(_) => update!(list_submitter),
        AppEvent::UpdateSubstituteSubmitter(ss) => update!(substitute_submitters, ss.id),
        AppEvent::UpdatePoliticalGroup(_) => update!(political_group),

        AppEvent::UpdatePersonPersonalData { person_id, .. }
        | AppEvent::UpdatePersonAddress { person_id, .. }
        | AppEvent::UpdatePersonRepresentative { person_id, .. } => update!(persons, person_id),
        AppEvent::UpdateCandidateListDistricts { list_id, .. }
        | AppEvent::UpdateCandidateListOrder { list_id, .. } => {
            update!(candidate_lists, list_id)
        }

        AppEvent::AddCandidateToCandidateList { person_id, .. } => {
            let new_val = serde_json::json!({ "person_id": person_id.to_string() });
            (None, Some(new_val))
        }
        AppEvent::RemoveCandidateFromCandidateList { person_id, .. } => {
            let old_val = serde_json::json!({ "person_id": person_id.to_string() });
            (Some(old_val), None)
        }

        AppEvent::DeletePerson { person_id } => update!(persons, person_id),
        AppEvent::DeleteCandidateList(cl_id) => update!(candidate_lists, cl_id),
        AppEvent::DeleteAuthorisedAgent(aa_id) => update!(authorised_agents, aa_id),
        AppEvent::DeleteSubstituteSubmitter {
            substitute_submitter_id: ss_id,
        } => update!(substitute_submitters, ss_id),

        AppEvent::DeveloperLogin { stream_id } => event!({ stream_id: stream_id.to_string() }),
        AppEvent::DownloadFile {
            file_name,
            download_path,
            list_id,
        } => event!({
            file_name: file_name,
            download_path: download_path,
            list_id: list_id.to_string(),
        }),
        AppEvent::ExportCsv {
            file_name,
            file_size,
            list_id,
        }
        | AppEvent::ImportCsv {
            file_name,
            file_size,
            list_id,
        } => event!({
            file_name: file_name,
            file_size: file_size,
            list_id: list_id.to_string(),
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        ElectionConfig, ElectoralDistrict, StreamId,
        authorised_agents::AuthorisedAgentId,
        candidate_lists::CandidateListId,
        list_submitters::ListSubmitterId,
        persons::PersonId,
        political_groups::PoliticalGroup,
        store::StoreData,
        test_utils::{
            sample_authorised_agent, sample_candidate_list, sample_list_submitter, sample_person,
            sample_political_group,
        },
    };

    fn empty_state() -> AppStoreData {
        AppStoreData::new(ElectionConfig::EK27)
    }

    // --- Create events: entity absent in `state_before`, present in `state_after` ---

    #[test]
    fn create_person_returns_none_and_new_from_state_after() {
        let person = sample_person(PersonId::new());
        let before = empty_state();
        let mut after = empty_state();
        after.persons.insert(person.id, person.clone());

        let (old, new) = extract_old_new(&AppEvent::CreatePerson(person.clone()), &before, &after);
        assert!(old.is_none());
        assert_eq!(new, serde_json::to_value(&person).ok());
    }

    #[test]
    fn create_candidate_list_pulls_new_from_state_after() {
        let list = sample_candidate_list(CandidateListId::new());
        let before = empty_state();
        let mut after = empty_state();
        after.candidate_lists.insert(list.id, list.clone());

        let (old, new) = extract_old_new(
            &AppEvent::CreateCandidateList(list.clone()),
            &before,
            &after,
        );
        assert!(old.is_none());
        assert_eq!(new, serde_json::to_value(&list).ok());
    }

    #[test]
    fn create_substitute_submitter_pulls_new_from_state_after() {
        let submitter = sample_list_submitter(ListSubmitterId::new());
        let before = empty_state();
        let mut after = empty_state();
        after
            .substitute_submitters
            .insert(submitter.id, submitter.clone());

        let (old, new) = extract_old_new(
            &AppEvent::CreateSubstituteSubmitter(submitter.clone()),
            &before,
            &after,
        );
        assert!(old.is_none());
        assert_eq!(new, serde_json::to_value(&submitter).ok());
    }

    #[test]
    fn create_authorised_agent_pulls_new_from_state_after() {
        let agent = sample_authorised_agent(AuthorisedAgentId::new());
        let before = empty_state();
        let mut after = empty_state();
        after.authorised_agents.insert(agent.id, agent.clone());

        let (old, new) = extract_old_new(
            &AppEvent::CreateAuthorisedAgent(agent.clone()),
            &before,
            &after,
        );
        assert!(old.is_none());
        assert_eq!(new, serde_json::to_value(&agent).ok());
    }

    // --- Update events: entity present in both snapshots, with different contents ---

    #[test]
    fn update_person_uses_state_after_for_new() {
        let person_id = PersonId::new();
        let before_person = sample_person(person_id);
        let mut after_person = before_person.clone();
        after_person.name.initials = "X.Y.".parse().unwrap();

        let mut before = empty_state();
        before.persons.insert(person_id, before_person.clone());
        let mut after = empty_state();
        after.persons.insert(person_id, after_person.clone());

        let (old, new) = extract_old_new(
            &AppEvent::UpdatePerson(after_person.clone()),
            &before,
            &after,
        );
        assert_eq!(old, serde_json::to_value(&before_person).ok());
        assert_eq!(new, serde_json::to_value(&after_person).ok());
    }

    #[test]
    fn update_person_personal_data_uses_full_person_snapshots() {
        let person_id = PersonId::new();
        let before_person = sample_person(person_id);
        let mut after_person = before_person.clone();
        after_person.personal_data.country = None;

        let mut before = empty_state();
        before.persons.insert(person_id, before_person.clone());
        let mut after = empty_state();
        after.persons.insert(person_id, after_person.clone());

        let (old, new) = extract_old_new(
            &AppEvent::UpdatePersonPersonalData {
                person_id,
                name: after_person.name.clone(),
                personal_data: after_person.personal_data.clone(),
            },
            &before,
            &after,
        );
        // The whole Person is serialized — this is the key change from the refactor:
        // we no longer build a custom { name, personal_data } payload.
        assert_eq!(old, serde_json::to_value(&before_person).ok());
        assert_eq!(new, serde_json::to_value(&after_person).ok());
    }

    #[test]
    fn update_political_group_uses_whole_field_with_no_id() {
        let mut before = empty_state();
        before.political_group = sample_political_group();
        let mut after = empty_state();
        after.political_group = PoliticalGroup {
            long_list_allowed: Some(true),
            ..sample_political_group()
        };

        let (old, new) = extract_old_new(
            &AppEvent::UpdatePoliticalGroup(after.political_group.clone()),
            &before,
            &after,
        );
        assert_eq!(old, serde_json::to_value(&before.political_group).ok());
        assert_eq!(new, serde_json::to_value(&after.political_group).ok());
    }

    #[test]
    fn update_candidate_list_districts_pulls_full_list_from_snapshots() {
        let list_id = CandidateListId::new();
        let mut before_list = sample_candidate_list(list_id);
        before_list.electoral_districts = vec![ElectoralDistrict::GR];
        let mut after_list = before_list.clone();
        after_list.electoral_districts = vec![ElectoralDistrict::GR, ElectoralDistrict::FR];

        let mut before = empty_state();
        before.candidate_lists.insert(list_id, before_list.clone());
        let mut after = empty_state();
        after.candidate_lists.insert(list_id, after_list.clone());

        let (old, new) = extract_old_new(
            &AppEvent::UpdateCandidateListDistricts {
                list_id,
                electoral_districts: after_list.electoral_districts.clone(),
            },
            &before,
            &after,
        );
        assert_eq!(old, serde_json::to_value(&before_list).ok());
        assert_eq!(new, serde_json::to_value(&after_list).ok());
    }

    // --- Delete events: entity present in `state_before`, absent in `state_after` ---

    #[test]
    fn delete_person_returns_old_from_state_before_and_none_new() {
        let person = sample_person(PersonId::new());
        let mut before = empty_state();
        before.persons.insert(person.id, person.clone());
        let after = empty_state();

        let (old, new) = extract_old_new(
            &AppEvent::DeletePerson {
                person_id: person.id,
            },
            &before,
            &after,
        );
        assert_eq!(old, serde_json::to_value(&person).ok());
        assert!(new.is_none());
    }

    #[test]
    fn delete_candidate_list_returns_old_from_state_before() {
        let list = sample_candidate_list(CandidateListId::new());
        let mut before = empty_state();
        before.candidate_lists.insert(list.id, list.clone());
        let after = empty_state();

        let (old, new) = extract_old_new(&AppEvent::DeleteCandidateList(list.id), &before, &after);
        assert_eq!(old, serde_json::to_value(&list).ok());
        assert!(new.is_none());
    }

    #[test]
    fn delete_of_missing_entity_yields_none_on_both_sides() {
        // If somehow both snapshots lack the entity, both sides are None —
        // this mirrors the behaviour of `.get().and_then(..)` on an absent key.
        let before = empty_state();
        let after = empty_state();
        let (old, new) = extract_old_new(
            &AppEvent::DeletePerson {
                person_id: PersonId::new(),
            },
            &before,
            &after,
        );
        assert!(old.is_none());
        assert_eq!(new, None);
    }

    // --- Add/Remove candidate: payload built directly, not from state ---

    #[test]
    fn add_candidate_returns_only_person_id_in_new() {
        let person_id = PersonId::new();
        let state = empty_state();
        let (old, new) = extract_old_new(
            &AppEvent::AddCandidateToCandidateList {
                list_id: CandidateListId::new(),
                person_id,
            },
            &state,
            &state,
        );
        assert!(old.is_none());
        assert_eq!(
            new,
            Some(serde_json::json!({ "person_id": person_id.to_string() }))
        );
    }

    #[test]
    fn remove_candidate_returns_only_person_id_in_old() {
        let person_id = PersonId::new();
        let state = empty_state();
        let (old, new) = extract_old_new(
            &AppEvent::RemoveCandidateFromCandidateList {
                list_id: CandidateListId::new(),
                person_id,
            },
            &state,
            &state,
        );
        assert_eq!(
            old,
            Some(serde_json::json!({ "person_id": person_id.to_string() }))
        );
        assert!(new.is_none());
    }

    // --- System events: payload is synthetic, independent of the app state ---

    #[test]
    fn developer_login_builds_event_payload_from_stream_id() {
        let stream_id = StreamId::new();
        let state = empty_state();
        let (old, new) = extract_old_new(&AppEvent::DeveloperLogin { stream_id }, &state, &state);
        assert!(old.is_none());
        assert_eq!(
            new,
            Some(serde_json::json!({ "stream_id": stream_id.to_string() }))
        );
    }

    #[test]
    fn download_file_builds_event_payload_with_all_fields() {
        let list_id = CandidateListId::new();
        let state = empty_state();
        let (old, new) = extract_old_new(
            &AppEvent::DownloadFile {
                file_name: "list.csv".to_string(),
                download_path: "/tmp/list.csv".to_string(),
                list_id,
            },
            &state,
            &state,
        );
        assert!(old.is_none());
        assert_eq!(
            new,
            Some(serde_json::json!({
                "file_name": "list.csv",
                "download_path": "/tmp/list.csv",
                "list_id": list_id.to_string(),
            }))
        );
    }

    #[test]
    fn export_and_import_csv_produce_identical_shape() {
        let list_id = CandidateListId::new();
        let state = empty_state();
        let export = AppEvent::ExportCsv {
            file_name: "x.csv".to_string(),
            file_size: 42,
            list_id,
        };
        let import = AppEvent::ImportCsv {
            file_name: "x.csv".to_string(),
            file_size: 42,
            list_id,
        };
        assert_eq!(
            extract_old_new(&export, &state, &state),
            extract_old_new(&import, &state, &state),
        );
    }
}
