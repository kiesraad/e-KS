use axum::Router;
use axum_extra::routing::{RouterExt, TypedPath};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    AppError, AppState, QueryParamState, StreamId,
    candidate_lists::CandidateListId,
    csb::examination::extractors::CsbPoliticalGroup,
    persons::PersonId,
    structs::csb::{OmissionId, OmissionType},
};

mod all_restorations;
mod candidate;
mod candidate_list;
mod correction;
mod general_information;
mod i4;
mod omission;
mod overview;
mod paper_corrections;
mod political_group;

#[derive(TypedPath)]
#[typed_path("/csb/examination", rejection(AppError))]
pub struct CsbExaminationOverviewPath;

#[derive(TypedPath)]
#[typed_path("/csb/examination/i4.pdf", rejection(AppError))]
pub struct CsbI4DownloadPath;

#[derive(TypedPath, Deserialize)]
#[typed_path("/csb/examination/{stream_id}", rejection(AppError))]
pub struct CsbPoliticalGroupPath {
    pub stream_id: StreamId,
}

#[derive(TypedPath, Deserialize)]
#[typed_path("/csb/examination/{stream_id}/toggle-finish", rejection(AppError))]
pub struct CsbPoliticalGroupToggleFinishPath {
    pub stream_id: StreamId,
}

#[derive(TypedPath, Deserialize)]
#[typed_path(
    "/csb/examination/{stream_id}/general-information",
    rejection(AppError)
)]
pub struct CsbGeneralInformationPath {
    pub stream_id: StreamId,
}

#[derive(TypedPath, Deserialize)]
#[typed_path("/csb/examination/{stream_id}/paper-corrections", rejection(AppError))]
pub struct CsbPaperCorrectionsStartPath {
    pub stream_id: StreamId,
}

#[derive(TypedPath, Deserialize)]
#[typed_path(
    "/csb/examination/{stream_id}/paper-corrections/stop",
    rejection(AppError)
)]
pub struct CsbPaperCorrectionsStopPath {
    pub stream_id: StreamId,
}

#[derive(TypedPath, Deserialize)]
#[typed_path("/csb/examination/{stream_id}/list/{list_id}", rejection(AppError))]
pub struct CsbCandidateListPath {
    pub stream_id: StreamId,
    pub list_id: CandidateListId,
}

#[derive(TypedPath, Deserialize)]
#[typed_path(
    "/csb/examination/{stream_id}/list/{list_id}/candidate/{person_id}",
    rejection(AppError)
)]
pub struct CsbCandidatePath {
    pub stream_id: StreamId,
    pub list_id: CandidateListId,
    pub person_id: PersonId,
}

#[derive(TypedPath, Deserialize)]
#[typed_path(
    "/csb/examination/{stream_id}/omission/{omission_type}/{reference}",
    rejection(AppError)
)]
pub struct CsbAddOmissionPath {
    pub stream_id: StreamId,
    pub omission_type: OmissionType,
    pub reference: Uuid,
}

#[derive(TypedPath, Deserialize)]
#[typed_path(
    "/csb/examination/{stream_id}/omission/{omission_type}/{reference}/overview",
    rejection(AppError)
)]
pub struct CsbOmissionOverviewPath {
    pub stream_id: StreamId,
    pub omission_type: OmissionType,
    pub reference: Uuid,
}

#[derive(TypedPath, Deserialize)]
#[typed_path(
    "/csb/examination/{stream_id}/delete-omission/{omission_id}",
    rejection(AppError)
)]
pub struct CsbDeleteOmissionPath {
    pub stream_id: StreamId,
    pub omission_id: OmissionId,
}

#[derive(TypedPath, Deserialize)]
#[typed_path("/csb/examination/{stream_id}/omissions", rejection(AppError))]
pub struct CsbAllRestorationsPath {
    pub stream_id: StreamId,
}

#[derive(TypedPath, Deserialize)]
#[typed_path(
    "/csb/examination/{stream_id}/correction/display-name",
    rejection(AppError)
)]
pub struct CsbDisplayNameCorrectionPath {
    pub stream_id: StreamId,
}

#[derive(TypedPath, Deserialize)]
#[typed_path(
    "/csb/examination/{stream_id}/correction/person/{person_id}/{field}",
    rejection(AppError)
)]
pub struct CsbPersonCorrectionPath {
    pub stream_id: StreamId,
    pub person_id: PersonId,
    pub field: CandidateCorrectionField,
}

/// Which personal-data field of a candidate the correction dialog operates on.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CandidateCorrectionField {
    Initials,
    LastName,
    DateOfBirth,
    PlaceOfResidence,
}

impl CandidateCorrectionField {
    pub fn label(self, locale: crate::Locale) -> String {
        match self {
            Self::Initials => crate::trans!("person.fields.initials", locale),
            Self::LastName => crate::trans!("person.fields.last_name", locale),
            Self::DateOfBirth => crate::trans!("person.fields.date_of_birth", locale),
            Self::PlaceOfResidence => crate::trans!("person.fields.place_of_residence", locale),
        }
    }
}

impl std::str::FromStr for CandidateCorrectionField {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "initials" => Ok(Self::Initials),
            "last-name" => Ok(Self::LastName),
            "date-of-birth" => Ok(Self::DateOfBirth),
            "place-of-residence" => Ok(Self::PlaceOfResidence),
            _ => Err("unknown correction field"),
        }
    }
}

impl std::fmt::Display for CandidateCorrectionField {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Initials => write!(f, "initials"),
            Self::LastName => write!(f, "last-name"),
            Self::DateOfBirth => write!(f, "date-of-birth"),
            Self::PlaceOfResidence => write!(f, "place-of-residence"),
        }
    }
}

impl<'de> Deserialize<'de> for CandidateCorrectionField {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let s = String::deserialize(deserializer)?;
        s.parse().map_err(serde::de::Error::custom)
    }
}

#[derive(Debug, Default, Clone, Copy, Serialize, Deserialize)]
pub struct OmissionListQuery {
    /// The candidate list the omission dialog was opened from. Used to resolve
    /// the candidate's position for the preset placeholders and to return to the
    /// candidate detail page, which is always scoped to a list.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub list: Option<CandidateListId>,
}

impl CsbPoliticalGroup {
    pub fn examination_path(&self) -> impl TypedPath {
        CsbPoliticalGroupPath {
            stream_id: self.stream_id,
        }
    }

    pub fn examination_toggle_finish_path(
        &self,
        redirect_to: impl std::fmt::Display,
    ) -> impl TypedPath {
        CsbPoliticalGroupToggleFinishPath {
            stream_id: self.stream_id,
        }
        .with_query_params(QueryParamState::redirect_to(redirect_to.to_string()))
    }

    pub fn general_information_path(&self) -> impl TypedPath {
        CsbGeneralInformationPath {
            stream_id: self.stream_id,
        }
    }

    /// Path that puts the session in paper-corrections mode for this stream.
    pub fn start_paper_corrections_path(&self) -> impl TypedPath {
        CsbPaperCorrectionsStartPath {
            stream_id: self.stream_id,
        }
    }

    /// Path to the dialog that adds a general (political group level) omission.
    pub fn add_political_group_omission_path(&self) -> impl TypedPath {
        CsbAddOmissionPath {
            stream_id: self.stream_id,
            omission_type: OmissionType::PoliticalGroup,
            reference: self.stream_id.into(),
        }
    }

    /// Path to the overview page listing the general (political group level)
    /// omissions already added.
    pub fn manage_political_group_omissions_path(&self) -> impl TypedPath {
        CsbOmissionOverviewPath {
            stream_id: self.stream_id,
            omission_type: OmissionType::PoliticalGroup,
            reference: self.stream_id.into(),
        }
    }

    /// Path to the dialog that adds an omission to a specific candidate list.
    pub fn add_candidate_list_omission_path(&self, list: &CandidateListId) -> impl TypedPath {
        CsbAddOmissionPath {
            stream_id: self.stream_id,
            omission_type: OmissionType::CandidateList,
            reference: (*list).into(),
        }
    }

    /// Path to the overview page listing the omissions already added to this
    /// candidate list.
    pub fn manage_candidate_list_omissions_path(&self, list: &CandidateListId) -> impl TypedPath {
        CsbOmissionOverviewPath {
            stream_id: self.stream_id,
            omission_type: OmissionType::CandidateList,
            reference: (*list).into(),
        }
    }

    /// Path to the candidate list examination page for a specific list.
    pub fn candidate_list_path(&self, list: &CandidateListId) -> impl TypedPath {
        CsbCandidateListPath {
            stream_id: self.stream_id,
            list_id: *list,
        }
    }

    /// Path to the detail page of a candidate examined on a specific list.
    pub fn candidate_path(&self, list: &CandidateListId, person: &PersonId) -> impl TypedPath {
        CsbCandidatePath {
            stream_id: self.stream_id,
            list_id: *list,
            person_id: *person,
        }
    }

    /// Path to the dialog that adds an omission to a candidate. The list is
    /// carried as a query parameter so the candidate's position on it can be
    /// resolved for the preset placeholders and to return to this page after.
    pub fn add_candidate_omission_path(
        &self,
        person: &PersonId,
        list: &CandidateListId,
    ) -> impl TypedPath {
        CsbAddOmissionPath {
            stream_id: self.stream_id,
            omission_type: OmissionType::Candidate,
            reference: (*person).into(),
        }
        .with_query_params(OmissionListQuery { list: Some(*list) })
    }

    /// Path to the overview page listing the omissions already added for this
    /// candidate (both list-scoped and general). The `list` is carried so the
    /// overview can link back to the add form and return page for the list.
    pub fn manage_candidate_omissions_path(
        &self,
        person: &PersonId,
        list: &CandidateListId,
    ) -> impl TypedPath {
        CsbOmissionOverviewPath {
            stream_id: self.stream_id,
            omission_type: OmissionType::Candidate,
            reference: (*person).into(),
        }
        .with_query_params(OmissionListQuery { list: Some(*list) })
    }

    pub fn all_restorations_path(&self) -> impl TypedPath {
        CsbAllRestorationsPath {
            stream_id: self.stream_id,
        }
    }

    /// Path to the correction overlay for the political group display name.
    pub fn correction_display_name_path(&self) -> impl TypedPath {
        CsbDisplayNameCorrectionPath {
            stream_id: self.stream_id,
        }
    }

    /// Path to the correction overlay for a specific personal-data field of a
    /// candidate. The `list` is carried as a query parameter so the overlay can
    /// return to the candidate's detail page after saving.
    pub fn correction_person_path(
        &self,
        person: &PersonId,
        field: CandidateCorrectionField,
        list: &CandidateListId,
    ) -> impl TypedPath {
        CsbPersonCorrectionPath {
            stream_id: self.stream_id,
            person_id: *person,
            field,
        }
        .with_query_params(OmissionListQuery { list: Some(*list) })
    }
}

pub fn router() -> Router<AppState> {
    Router::new()
        .typed_get(overview::overview)
        .typed_get(i4::gen_i4)
        .typed_get(political_group::overview)
        .typed_post(political_group::toggle_examination_finish)
        .typed_get(general_information::overview)
        .typed_post(paper_corrections::start_paper_corrections)
        .typed_post(paper_corrections::stop_paper_corrections)
        .typed_get(candidate_list::overview)
        .typed_get(candidate::overview)
        .typed_get(omission::add_omission)
        .typed_post(omission::add_omission_submit)
        .typed_get(omission::overview)
        .typed_post(omission::delete_omission)
        .typed_get(all_restorations::all_restorations)
        .typed_get(correction::display_name_correction)
        .typed_post(correction::display_name_correction_submit)
        .typed_get(correction::person_correction)
        .typed_post(correction::person_correction_submit)
}
