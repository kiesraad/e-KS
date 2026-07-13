use askama::Template;
use axum_extra::routing::TypedPath;

use crate::{
    Context, CsbStore, Locale, Overlay, QueryParamState,
    candidate_lists::CandidateListId,
    csb::{
        Omission, OmissionPlaceholders, OmissionType,
        examination::{OmissionForm, pages::CsbDeleteOmissionPath},
    },
    filters,
    form::FormData,
    persons::PersonId,
};

use super::OmissionTarget;

/// The add-omission form tab of the dialog.
#[derive(Template)]
#[template(path = "csb/examination/pages/omission.html")]
pub(super) struct CsbAddOmissionTemplate {
    pub(super) form: FormData<OmissionForm>,
    pub(super) overlay: Overlay,
    /// Where the close button and the post-save redirect return to.
    pub(super) close_action: String,
    /// Quick-fill suggestions for this type, with placeholders interpolated.
    pub(super) presets: Vec<PresetView>,
    /// The dialog opened on its two tabs, for the steps sidebar.
    pub(super) add_tab_url: String,
    pub(super) overview_tab_url: String,
}

/// The overview tab of the dialog: the omissions already added to this entity.
#[derive(Template)]
#[template(path = "csb/examination/pages/omission_overview.html")]
pub(super) struct CsbOmissionOverviewTemplate {
    pub(super) overlay: Overlay,
    /// Where the close button returns to.
    pub(super) close_action: String,
    /// The omissions already added to this entity, each with a remove action.
    pub(super) omissions: Vec<OmissionView>,
    /// The dialog opened on its two tabs, for the steps sidebar.
    pub(super) add_tab_url: String,
    pub(super) overview_tab_url: String,
}

/// An omission in the overview tab, paired with the URL of its remove action
/// (which returns to this same overview afterwards).
pub(super) struct OmissionView {
    omission: Omission,
    remove_url: String,
}

/// A preset shown in the dialog, with `{token}` placeholders in its description
/// already filled from the referenced item (the rest left for manual entry).
pub(super) struct PresetView {
    title: String,
    description: String,
    help_text: String,
    /// Whether this preset describes a recoverable omission ("herstelbaar").
    recoverable: bool,
}

/// Resolve the placeholder values that can be derived from the referenced item.
fn placeholders_for(
    target: &OmissionTarget,
    store: &CsbStore,
    locale: Locale,
) -> OmissionPlaceholders {
    match target.omission_type {
        OmissionType::Candidate => {
            let person = PersonId::from(target.reference);
            OmissionPlaceholders {
                candidate_name: store.get_person(person).map(|person| person.name.display()),
                // A candidate's position differs per list, so it can only be
                // resolved when the dialog was opened for a specific list.
                candidate_number: target
                    .list
                    .and_then(|list| store.candidate_position(list, person))
                    .map(|nr| nr.to_string()),
                districts: None,
            }
        }
        OmissionType::CandidateList => OmissionPlaceholders {
            districts: store
                .get_candidate_list(CandidateListId::from(target.reference))
                .map(|list| list.districts_name(locale.into())),
            ..Default::default()
        },
        OmissionType::PoliticalGroup => OmissionPlaceholders::default(),
    }
}

/// The presets for this type with their descriptions interpolated. A `general`
/// candidate omission (applying to the person on every list) offers a different
/// set than one scoped to the candidate on a specific list.
pub(super) fn preset_views(
    target: &OmissionTarget,
    store: &CsbStore,
    locale: Locale,
) -> Vec<PresetView> {
    let placeholders = placeholders_for(target, store, locale);

    target
        .omission_type
        .presets(target.general)
        .iter()
        .map(|preset| PresetView {
            title: preset.title.clone(),
            description: placeholders.interpolate(&preset.description),
            help_text: preset.help_text.clone(),
            recoverable: preset.recoverable,
        })
        .collect()
}

/// The omissions already added to the entity the dialog was opened for, shown
/// on the overview tab. A candidate lists every omission for the person, both
/// list-scoped and general. Each is paired with a remove action that returns to
/// `overview_url` afterwards.
pub(super) fn omission_views(
    target: &OmissionTarget,
    store: &CsbStore,
    overview_url: &str,
) -> Vec<OmissionView> {
    let omissions = match target.omission_type {
        OmissionType::PoliticalGroup => store.get_general_omissions(),
        OmissionType::CandidateList => {
            store.get_candidate_list_omissions(CandidateListId::from(target.reference))
        }
        OmissionType::Candidate => store.get_candidate_omissions(PersonId::from(target.reference)),
    };

    omissions
        .into_iter()
        .map(|omission| OmissionView {
            remove_url: CsbDeleteOmissionPath {
                stream_id: target.stream_id,
                omission_id: omission.id,
            }
            .with_query_params(QueryParamState::redirect_to(overview_url.to_string()))
            .to_string(),
            omission,
        })
        .collect()
}
