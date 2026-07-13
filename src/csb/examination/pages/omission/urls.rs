use axum_extra::routing::TypedPath;

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

use super::OmissionTarget;

/// The page the dialog returns to: the general information page for political
/// group omissions, the candidate list page for candidate list omissions, the
/// candidate detail page for candidate omissions opened from a specific list,
/// otherwise the political group examination overview.
pub(super) fn return_path(target: &OmissionTarget, political_group: &CsbPoliticalGroup) -> String {
    match target.omission_type {
        OmissionType::PoliticalGroup => political_group.general_information_path().to_string(),
        OmissionType::CandidateList => political_group
            .candidate_list_path(&CandidateListId::from(target.reference))
            .to_string(),
        // The candidate detail page is scoped to a list, so it can only be the
        // return target when the dialog was opened for a specific list.
        OmissionType::Candidate => match target.list {
            Some(list) => political_group
                .candidate_path(&list, &PersonId::from(target.reference))
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
pub(super) fn add_url(target: &OmissionTarget) -> String {
    with_context(
        CsbAddOmissionPath {
            stream_id: target.stream_id,
            omission_type: target.omission_type,
            reference: target.reference,
        },
        target.list,
        target.general,
    )
}

/// The URL of the overview page for this entity, keeping the list/general
/// context (the sidebar links here from the add form).
pub(super) fn overview_url(target: &OmissionTarget) -> String {
    with_context(
        CsbOmissionOverviewPath {
            stream_id: target.stream_id,
            omission_type: target.omission_type,
            reference: target.reference,
        },
        target.list,
        target.general,
    )
}

/// Fallback overview URL to return to after removing an omission, derived from
/// its category. Used only when the request carries no explicit `redirect_to`.
pub(super) fn overview_url_for(category: &OmissionCategory, stream_id: StreamId) -> String {
    let target = match category {
        OmissionCategory::CandidateList(id) => OmissionTarget {
            stream_id,
            omission_type: OmissionType::CandidateList,
            reference: (*id).into(),
            list: None,
            general: false,
        },
        OmissionCategory::Candidate { person, list } => OmissionTarget {
            stream_id,
            omission_type: OmissionType::Candidate,
            reference: (*person).into(),
            list: *list,
            general: list.is_none(),
        },
        _ => OmissionTarget {
            stream_id,
            omission_type: OmissionType::PoliticalGroup,
            reference: stream_id.into(),
            list: None,
            general: false,
        },
    };
    overview_url(&target)
}
