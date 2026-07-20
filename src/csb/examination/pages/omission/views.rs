use askama::Template;
use axum_extra::routing::TypedPath;

use crate::{
    AppError, Context, CsbStore, ElectoralDistrict, Locale, Overlay, QueryParamState,
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

/// One selectable candidate list in the add-omission dialog for candidate omissions.
pub(super) struct CandidateListOption {
    pub(super) id: CandidateListId,
    /// Dutch label derived from the list's electoral districts (e.g. "1. Groningen").
    pub(super) label: String,
}

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
    /// Districts that appear on at least one candidate list of this political
    /// group. The districts section is hidden when this is empty. Districts
    /// absent from all lists are shown disabled so the user cannot select them.
    pub(super) available_districts: Vec<ElectoralDistrict>,
    /// Candidate lists for a Candidate omission. Hidden when empty.
    pub(super) available_candidate_lists: Vec<CandidateListOption>,
    pub(super) title_suffix: String,
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
    pub(super) title_suffix: String,
}

/// An omission in the overview tab, paired with the URL of its remove action
/// (which returns to this same overview afterwards).
pub(super) struct OmissionView {
    omission: Omission,
    remove_url: String,
    /// Formatted district string for display (e.g. "1. Groningen, 2. Friesland").
    districts: String,
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
fn placeholders_for(target: &OmissionTarget, store: &CsbStore) -> OmissionPlaceholders {
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
            }
        }
        // The {district}/{districts} tokens in candidate-list presets are filled
        // in by the front-end
        OmissionType::CandidateList | OmissionType::PoliticalGroup => {
            OmissionPlaceholders::default()
        }
    }
}

/// All candidate lists of the political group for the candidate omission form
pub(super) fn candidate_list_options(store: &CsbStore, locale: Locale) -> Vec<CandidateListOption> {
    store
        .get_candidate_lists()
        .into_iter()
        .map(|l| CandidateListOption {
            id: l.id,
            label: l.districts_name(locale.into()),
        })
        .collect()
}

/// The presets for this type with their descriptions interpolated.
pub(super) fn preset_views(target: &OmissionTarget, store: &CsbStore) -> Vec<PresetView> {
    let placeholders = placeholders_for(target, store);

    target
        .omission_type
        .presets()
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
) -> Result<Vec<OmissionView>, AppError> {
    let omissions = match target.omission_type {
        OmissionType::PoliticalGroup => store.get_political_group_omissions(),
        OmissionType::CandidateList => {
            store.get_candidate_list_omissions(CandidateListId::from(target.reference))?
        }
        OmissionType::Candidate => store.get_candidate_omissions(PersonId::from(target.reference)),
    };

    let mut views = Vec::with_capacity(omissions.len());
    for omission in omissions {
        let districts = omission
            .category
            .electoral_district(store, &store.election)?;
        views.push(OmissionView {
            remove_url: CsbDeleteOmissionPath {
                stream_id: target.stream_id,
                omission_id: omission.id,
            }
            .with_query_params(QueryParamState::redirect_to(overview_url.to_string()))
            .to_string(),
            omission,
            districts,
        });
    }
    Ok(views)
}
