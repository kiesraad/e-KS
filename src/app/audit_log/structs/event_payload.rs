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
        AppEvent::CreateListSubmitter(ls) => update!(list_submitters, ls.id),
        AppEvent::CreateSubstituteSubmitter(ss) => update!(substitute_submitters, ss.id),

        AppEvent::UpdatePerson(person) => update!(persons, person.id),
        AppEvent::UpdateAuthorisedAgent(aa) => update!(authorised_agents, aa.id),
        AppEvent::UpdateListSubmitter(ls) => update!(list_submitters, ls.id),
        AppEvent::UpdateSubstituteSubmitter(ss) => update!(substitute_submitters, ss.id),
        AppEvent::UpdatePoliticalGroup(_) => update!(political_group),

        AppEvent::UpdatePersonPersonalData { person_id, .. }
        | AppEvent::UpdatePersonAddress { person_id, .. }
        | AppEvent::UpdatePersonRepresentative { person_id, .. } => update!(persons, person_id),
        AppEvent::UpdateCandidateListDistricts { list_id, .. }
        | AppEvent::UpdateCandidateListOrder { list_id, .. }
        | AppEvent::UpdateCandidateListSubmitters { list_id, .. } => {
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
        AppEvent::DeleteListSubmitter {
            list_submitter_id: ls_id,
        } => update!(list_submitters, ls_id),
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
