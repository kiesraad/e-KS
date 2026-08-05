use askama::Template;
use axum_extra::routing::TypedPath;

use crate::{
    AppError, Context, CsbStore, ElectoralDistrict, Locale, Overlay,
    candidate_lists::CandidateListId,
    csb::{
        WithCorrections,
        examination::{OmissionForm, pages::CsbDeleteOmissionPath},
    },
    filters,
    form::FormData,
    persons::PersonId,
    structs::csb::{Omission, OmissionPlaceholders, OmissionType},
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
    /// target to generate urls from, for the steps sidebar.
    pub(super) omission_target: OmissionTarget,
    /// Districts that appear on at least one paper-corrected candidate list of
    /// this political group. The districts section is hidden when this is
    /// empty. Districts absent from all lists are shown disabled so the user
    /// cannot select them.
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
    /// target to generate urls from, for the steps sidebar.
    pub(super) omission_target: OmissionTarget,
    pub(super) title_suffix: String,
}

/// An omission in the overview tab; the URL of its remove action is derived
/// from the dialog's target via `remove_url`.
pub(super) struct OmissionView {
    omission: Omission,
    /// Formatted district string for display (e.g. "1. Groningen, 2. Friesland").
    districts: String,
}

impl OmissionView {
    fn remove_url(&self, omission_target: &OmissionTarget) -> impl TypedPath {
        CsbDeleteOmissionPath {
            stream_id: omission_target.stream_id,
            omission_id: self.omission.id,
        }
    }
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
                candidate_name: store
                    .get_person(person, WithCorrections::All)
                    .map(|person| person.name.display()),
                // A candidate's position differs per list, so it can only be
                // resolved when the dialog was opened for a specific list.
                candidate_number: target
                    .list
                    .and_then(|list| {
                        store.get_candidate_position(list, person, WithCorrections::All)
                    })
                    .map(|nr| nr.to_string()),
            }
        }
        // The {district}/{districts} tokens in candidate-list presets are filled
        // in by the front-end
        OmissionType::CandidateList
        | OmissionType::DeclarationsOfSupport
        | OmissionType::PoliticalGroup => OmissionPlaceholders::default(),
    }
}

/// All paper-corrected candidate lists of the political group for the
/// candidate omission form
pub(super) fn candidate_list_options(store: &CsbStore, locale: Locale) -> Vec<CandidateListOption> {
    store
        .get_candidate_lists(WithCorrections::All)
        .into_iter()
        .map(|l| CandidateListOption {
            id: l.id,
            label: l.districts_name(locale.into()),
        })
        .collect()
}

/// All paper-corrected candidate list districts of the political group
/// for the candidate list omission form (mainly declarations of support)
pub(super) fn available_electoral_districts(store: &CsbStore) -> Vec<ElectoralDistrict> {
    let mut districts: Vec<_> = store
        .get_candidate_lists(WithCorrections::All)
        .into_iter()
        .flat_map(|l| l.electoral_districts)
        .collect();
    districts.sort();
    districts.dedup();
    districts
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
/// list-scoped and general.
pub(super) fn omission_views(
    target: &OmissionTarget,
    store: &CsbStore,
) -> Result<Vec<OmissionView>, AppError> {
    let omissions = match target.omission_type {
        OmissionType::PoliticalGroup => store.get_political_group_omissions(),
        OmissionType::CandidateList => {
            store.get_candidate_list_omissions(CandidateListId::from(target.reference))?
        }
        OmissionType::DeclarationsOfSupport => store.get_all_declarations_of_support_omissions(),
        OmissionType::Candidate => store.get_candidate_omissions(PersonId::from(target.reference)),
    };

    let mut views = Vec::with_capacity(omissions.len());
    for omission in omissions {
        let districts = omission
            .category
            .electoral_district(store, &store.election)?;
        views.push(OmissionView {
            omission,
            districts,
        });
    }
    Ok(views)
}
