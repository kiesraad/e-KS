use axum_extra::routing::TypedPath;
use uuid::Uuid;

use crate::{
    StreamId,
    candidate_lists::CandidateListId,
    csb::{
        OmissionCategory, OmissionType,
        examination::{
            extractors::CsbPoliticalGroup,
            pages::{CsbAddOmissionPath, CsbOmissionOverviewPath, OmissionListQuery},
        },
    },
    persons::PersonId,
};

/// The page the dialog returns to: the general information page for political
/// group omissions, the candidate list page for candidate list omissions, the
/// candidate detail page for candidate omissions opened from a specific list,
/// otherwise the political group examination overview.
pub(super) fn return_path(
    omission_type: OmissionType,
    reference: Uuid,
    list: Option<CandidateListId>,
    political_group: &CsbPoliticalGroup,
) -> String {
    match omission_type {
        OmissionType::PoliticalGroup => political_group.general_information_path().to_string(),
        OmissionType::CandidateList => political_group
            .candidate_list_path(&CandidateListId::from(reference))
            .to_string(),
        // The candidate detail page is scoped to a list, so it can only be the
        // return target when the dialog was opened for a specific list.
        OmissionType::Candidate => match list {
            Some(list) => political_group
                .candidate_path(&list, &PersonId::from(reference))
                .to_string(),
            None => political_group.examination_path().to_string(),
        },
    }
}

/// Append the list/general context as a query string, but only when there is
/// something to carry (an empty query would otherwise leave a trailing `?`).
fn with_context(path: impl TypedPath, list: Option<CandidateListId>, general: bool) -> String {
    if list.is_none() && !general {
        path.to_string()
    } else {
        path.with_query_params(OmissionListQuery { list, general })
            .to_string()
    }
}

/// The URL of the add-omission form for this entity, keeping the list/general
/// context (the sidebar links here from the overview page).
pub(super) fn add_url(
    stream_id: StreamId,
    omission_type: OmissionType,
    reference: Uuid,
    list: Option<CandidateListId>,
    general: bool,
) -> String {
    with_context(
        CsbAddOmissionPath {
            stream_id,
            omission_type,
            reference,
        },
        list,
        general,
    )
}

/// The URL of the overview page for this entity, keeping the list/general
/// context (the sidebar links here from the add form).
pub(super) fn overview_url(
    stream_id: StreamId,
    omission_type: OmissionType,
    reference: Uuid,
    list: Option<CandidateListId>,
    general: bool,
) -> String {
    with_context(
        CsbOmissionOverviewPath {
            stream_id,
            omission_type,
            reference,
        },
        list,
        general,
    )
}

/// Fallback overview URL to return to after removing an omission, derived from
/// its category. Used only when the request carries no explicit `redirect_to`.
pub(super) fn overview_url_for(category: &OmissionCategory, stream_id: StreamId) -> String {
    match category {
        OmissionCategory::CandidateList(id) => overview_url(
            stream_id,
            OmissionType::CandidateList,
            (*id).into(),
            None,
            false,
        ),
        OmissionCategory::Candidate { person, list } => overview_url(
            stream_id,
            OmissionType::Candidate,
            (*person).into(),
            *list,
            list.is_none(),
        ),
        _ => overview_url(
            stream_id,
            OmissionType::PoliticalGroup,
            stream_id.into(),
            None,
            false,
        ),
    }
}
