//! Extract the old/new JSON snapshots for a given event, used by
//! `AuditLogDetail::compute` to build a field-level diff.
//!
//! - Create events: no old state; new contains the created entity.
//! - Update events: old is the prior entity from `state_before`; new is from
//!   `state_after`.
//! - Delete events: old is the entity from `state_before`; no new state.
//! - System events: informational only; rendered as additions.

use crate::{AppEvent, AppStoreData, persons::Person};

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

        (Vec, $kind:ident, $id:expr) => {{
            let old = state_before
                .$kind
                .iter()
                .find(|data| data.id == $id)
                .and_then(|data| serde_json::to_value(data).ok());
            let new = state_after
                .$kind
                .iter()
                .find(|data| data.id == $id)
                .and_then(|data| serde_json::to_value(data).ok());
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
        AppEvent::CreateNameAuthorisation(na) => update!(name_authorisations, na.id),
        AppEvent::CreateSubstituteSubmitter(ss) => update!(Vec, substitute_submitters, ss.id),

        AppEvent::UpdatePerson(person) => update!(persons, person.id),
        AppEvent::UpdateNameAuthorisation(na) => update!(name_authorisations, na.id),
        AppEvent::UpdateListSubmitter(_) => update!(list_submitter),
        AppEvent::UpdateSubstituteSubmitter(ss) => update!(Vec, substitute_submitters, ss.id),
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
        AppEvent::DeleteNameAuthorisation(na_id) => update!(name_authorisations, na_id),
        AppEvent::DeleteSubstituteSubmitter {
            substitute_submitter_id: ss_id,
        } => update!(Vec, substitute_submitters, *ss_id),

        AppEvent::DeveloperLogin { stream_id } => event!({ stream_id: stream_id.to_string() }),
        AppEvent::HideDownloadWarning | AppEvent::Import { .. } => event!({}),
        AppEvent::DownloadFile {
            file_name,
            download_path,
        } => event!({
            file_name: file_name,
            download_path: download_path,
        }),
        AppEvent::ExportCsv {
            file_name,
            file_size,
            list_id,
        } => event!({
            file_name: file_name,
            file_size: file_size,
            list_id: list_id.to_string(),
        }),
        AppEvent::ImportCandidates {
            file_name,
            file_size,
            list_id,
            created_persons,
            updated_persons,
            ..
        } => {
            fn person_ids(persons: &[Person]) -> Vec<String> {
                persons.iter().map(|p| p.id.to_string()).collect()
            }
            event!({
                file_name: file_name,
                file_size: file_size,
                list_id: list_id.to_string(),
                created_persons: person_ids(created_persons),
                updated_persons: person_ids(updated_persons),
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        ElectoralDistrict, StreamId,
        candidate_lists::CandidateListId,
        common::PreviousElectionResults,
        list_submitters::ListSubmitterId,
        name_authorisations::NameAuthorisationId,
        persons::PersonId,
        test_utils::{
            sample_candidate_list, sample_list_submitter, sample_name_authorisation, sample_person,
            sample_political_group,
        },
    };

    fn empty_state() -> AppStoreData {
        AppStoreData::default()
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
        after.substitute_submitters.push(submitter.clone());

        let (old, new) = extract_old_new(
            &AppEvent::CreateSubstituteSubmitter(submitter.clone()),
            &before,
            &after,
        );
        assert!(old.is_none());
        assert_eq!(new, serde_json::to_value(&submitter).ok());
    }

    #[test]
    fn create_name_authorisation_pulls_new_from_state_after() {
        let name_auth = sample_name_authorisation(NameAuthorisationId::new());
        let before = empty_state();
        let mut after = empty_state();
        after
            .name_authorisations
            .insert(name_auth.id, name_auth.clone());

        let (old, new) = extract_old_new(
            &AppEvent::CreateNameAuthorisation(name_auth.clone()),
            &before,
            &after,
        );
        assert!(old.is_none());
        assert_eq!(new, serde_json::to_value(&name_auth).ok());
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
        after.political_group = sample_political_group();
        after.political_group.previous_election_results =
            Some(PreviousElectionResults::SixteenOrMoreSeats);

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
        let state = empty_state();
        let (old, new) = extract_old_new(
            &AppEvent::DownloadFile {
                file_name: "list.csv".to_string(),
                download_path: "/tmp/list.csv".to_string(),
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
            }))
        );
    }

    #[test]
    fn import_candidates_payload_includes_imported_persons() {
        let list_id = CandidateListId::new();
        let state = empty_state();
        let person_a = sample_person(PersonId::new());
        let person_b = sample_person(PersonId::new());
        let person_c = sample_person(PersonId::new());

        let event = AppEvent::ImportCandidates {
            list_id,
            file_name: "candidates.csv".to_string(),
            file_size: 123,
            created_persons: vec![person_a.clone(), person_b.clone()],
            updated_persons: vec![person_c.clone()],
            candidates: vec![person_a.id, person_b.id, person_c.id],
        };

        let (old, new) = extract_old_new(&event, &state, &state);

        assert!(old.is_none());
        assert_eq!(
            new,
            Some(serde_json::json!({
                "file_name": "candidates.csv",
                "file_size": 123,
                "list_id": list_id.to_string(),
                "created_persons": [
                    person_a.id.to_string(),
                    person_b.id.to_string(),
                ],
                "updated_persons": [person_c.id.to_string()],
            }))
        );
    }
}
