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
    macro_rules! add {
        ($data:expr) => {
            (None, serde_json::to_value($data).ok())
        };
    }

    macro_rules! update {
        ($kind:ident, $data:expr) => {{
            let old = state_before
                .$kind
                .get(&$data.id)
                .and_then(|data| serde_json::to_value(data).ok());
            (old, serde_json::to_value($data).ok())
        }};
    }

    macro_rules! delete {
        ($kind:ident, $id:expr) => {{
            let old = state_before
                .$kind
                .get($id)
                .and_then(|data| serde_json::to_value(data).ok());
            (old, None)
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
        AppEvent::CreatePerson(person) => add!(person),
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
        AppEvent::CreateCandidateList(cl) => add!(cl),
        AppEvent::CreateAuthorisedAgent(aa) => add!(aa),
        AppEvent::CreateListSubmitter(ls) => add!(ls),
        AppEvent::CreateSubstituteSubmitter(ss) => add!(ss),

        AppEvent::UpdatePerson(person) => update!(persons, person),
        AppEvent::UpdateAuthorisedAgent(aa) => update!(authorised_agents, aa),
        AppEvent::UpdateListSubmitter(ls) => update!(list_submitters, ls),
        AppEvent::UpdateSubstituteSubmitter(ss) => update!(substitute_submitters, ss),
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

        AppEvent::DeletePerson { person_id } => delete!(persons, person_id),
        AppEvent::DeleteCandidateList(cl_id) => delete!(candidate_lists, cl_id),
        AppEvent::DeleteAuthorisedAgent(aa_id) => delete!(authorised_agents, aa_id),
        AppEvent::DeleteListSubmitter {
            list_submitter_id: ls_id,
        } => delete!(list_submitters, ls_id),
        AppEvent::DeleteSubstituteSubmitter {
            substitute_submitter_id: ss_id,
        } => delete!(substitute_submitters, ss_id),

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
