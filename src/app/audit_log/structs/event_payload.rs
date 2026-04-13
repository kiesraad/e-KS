//! Extract the old/new JSON snapshots for a given event, used by
//! `AuditLogDetail::compute` to build a field-level diff.
//!
//! - Create events: no old state; new contains the created entity.
//! - Update events: old is the prior entity from `state_before`; new is the
//!   updated payload.
//! - Delete events: old is the entity from `state_before`; no new state.
//! - System events: informational only; rendered as additions.

use crate::{AppEvent, AppStoreData};

pub(super) fn extract_old_new(
    event: &AppEvent,
    state_before: &AppStoreData,
) -> (Option<serde_json::Value>, Option<serde_json::Value>) {
    match event {
        AppEvent::CreatePerson(person) => (None, serde_json::to_value(person).ok()),
        AppEvent::CreatePersonPersonalData {
            person_id: _,
            name,
            personal_data,
        } => {
            let new_val = serde_json::json!({
                "name": serde_json::to_value(name).unwrap_or_default(),
                "personal_data": serde_json::to_value(personal_data).unwrap_or_default(),
            });
            (None, Some(new_val))
        }
        AppEvent::CreateCandidateList(cl) => (None, serde_json::to_value(cl).ok()),
        AppEvent::CreateAuthorisedAgent(aa) => (None, serde_json::to_value(aa).ok()),
        AppEvent::CreateListSubmitter(ls) => (None, serde_json::to_value(ls).ok()),
        AppEvent::CreateSubstituteSubmitter(ss) => (None, serde_json::to_value(ss).ok()),

        AppEvent::UpdatePerson(person) => {
            let old = state_before
                .persons
                .get(&person.id)
                .and_then(|p| serde_json::to_value(p).ok());
            (old, serde_json::to_value(person).ok())
        }
        AppEvent::UpdateAuthorisedAgent(aa) => {
            let old = state_before
                .authorised_agents
                .get(&aa.id)
                .and_then(|a| serde_json::to_value(a).ok());
            (old, serde_json::to_value(aa).ok())
        }
        AppEvent::UpdateListSubmitter(ls) => {
            let old = state_before
                .list_submitters
                .get(&ls.id)
                .and_then(|l| serde_json::to_value(l).ok());
            (old, serde_json::to_value(ls).ok())
        }
        AppEvent::UpdateSubstituteSubmitter(ss) => {
            let old = state_before
                .substitute_submitters
                .get(&ss.id)
                .and_then(|s| serde_json::to_value(s).ok());
            (old, serde_json::to_value(ss).ok())
        }
        AppEvent::UpdatePoliticalGroup(pg) => {
            let old = serde_json::to_value(&state_before.political_group).ok();
            (old, serde_json::to_value(pg).ok())
        }

        AppEvent::UpdatePersonPersonalData {
            person_id,
            name,
            personal_data,
        } => {
            let old = state_before.persons.get(person_id).map(|p| {
                serde_json::json!({
                    "name": serde_json::to_value(&p.name).unwrap_or_default(),
                    "personal_data": serde_json::to_value(&p.personal_data).unwrap_or_default(),
                })
            });
            let new_val = serde_json::json!({
                "name": serde_json::to_value(name).unwrap_or_default(),
                "personal_data": serde_json::to_value(personal_data).unwrap_or_default(),
            });
            (old, Some(new_val))
        }
        AppEvent::UpdatePersonAddress { person_id, address } => {
            let old = state_before
                .persons
                .get(person_id)
                .and_then(|p| serde_json::to_value(&p.address).ok());
            (old, serde_json::to_value(address).ok())
        }
        AppEvent::UpdatePersonRepresentative {
            person_id,
            representative,
        } => {
            let old = state_before
                .persons
                .get(person_id)
                .and_then(|p| serde_json::to_value(&p.representative).ok());
            (old, serde_json::to_value(representative).ok())
        }
        AppEvent::UpdateCandidateListDistricts {
            list_id,
            electoral_districts,
        } => {
            let old = state_before.candidate_lists.get(list_id).map(|cl| {
                serde_json::json!({
                    "electoral_districts": serde_json::to_value(&cl.electoral_districts).unwrap_or_default(),
                })
            });
            let new_val = serde_json::json!({
                "electoral_districts": serde_json::to_value(electoral_districts).unwrap_or_default(),
            });
            (old, Some(new_val))
        }
        AppEvent::UpdateCandidateListOrder {
            list_id,
            candidates,
        } => {
            let old = state_before.candidate_lists.get(list_id).map(|cl| {
                serde_json::json!({
                    "candidates": serde_json::to_value(&cl.candidates).unwrap_or_default(),
                })
            });
            let new_val = serde_json::json!({
                "candidates": serde_json::to_value(candidates).unwrap_or_default(),
            });
            (old, Some(new_val))
        }
        AppEvent::UpdateCandidateListSubmitters {
            list_id,
            list_submitter_id,
            substitute_list_submitter_ids,
        } => {
            let old = state_before.candidate_lists.get(list_id).map(|cl| {
                serde_json::json!({
                    "list_submitter_id": serde_json::to_value(cl.list_submitter_id).unwrap_or_default(),
                    "substitute_list_submitter_ids": serde_json::to_value(&cl.substitute_list_submitter_ids).unwrap_or_default(),
                })
            });
            let new_val = serde_json::json!({
                "list_submitter_id": serde_json::to_value(list_submitter_id).unwrap_or_default(),
                "substitute_list_submitter_ids": serde_json::to_value(substitute_list_submitter_ids).unwrap_or_default(),
            });
            (old, Some(new_val))
        }

        AppEvent::AddCandidateToCandidateList { person_id, .. } => {
            let new_val = serde_json::json!({ "person_id": person_id.to_string() });
            (None, Some(new_val))
        }
        AppEvent::RemoveCandidateFromCandidateList { person_id, .. } => {
            let old_val = serde_json::json!({ "person_id": person_id.to_string() });
            (Some(old_val), None)
        }

        AppEvent::DeletePerson { person_id } => {
            let old = state_before
                .persons
                .get(person_id)
                .and_then(|p| serde_json::to_value(p).ok());
            (old, None)
        }
        AppEvent::DeleteCandidateList(cl_id) => {
            let old = state_before
                .candidate_lists
                .get(cl_id)
                .and_then(|cl| serde_json::to_value(cl).ok());
            (old, None)
        }
        AppEvent::DeleteAuthorisedAgent(aa_id) => {
            let old = state_before
                .authorised_agents
                .get(aa_id)
                .and_then(|a| serde_json::to_value(a).ok());
            (old, None)
        }
        AppEvent::DeleteListSubmitter {
            list_submitter_id: ls_id,
        } => {
            let old = state_before
                .list_submitters
                .get(ls_id)
                .and_then(|l| serde_json::to_value(l).ok());
            (old, None)
        }
        AppEvent::DeleteSubstituteSubmitter {
            substitute_submitter_id: ss_id,
        } => {
            let old = state_before
                .substitute_submitters
                .get(ss_id)
                .and_then(|s| serde_json::to_value(s).ok());
            (old, None)
        }

        AppEvent::DeveloperLogin { stream_id } => {
            let val = serde_json::json!({ "stream_id": stream_id.to_string() });
            (None, Some(val))
        }
        AppEvent::DownloadFile {
            file_name,
            download_path,
            list_id,
        } => {
            let val = serde_json::json!({
                "file_name": file_name,
                "download_path": download_path,
                "list_id": list_id.to_string(),
            });
            (None, Some(val))
        }
        AppEvent::ExportCsv {
            file_name,
            file_size,
            list_id,
        }
        | AppEvent::ImportCsv {
            file_name,
            file_size,
            list_id,
        } => {
            let val = serde_json::json!({
                "file_name": file_name,
                "file_size": file_size,
                "list_id": list_id.to_string(),
            });
            (None, Some(val))
        }
    }
}
