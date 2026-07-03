use serde::{Deserialize, Serialize};

use crate::{
    ElectoralDistrict, Event, StreamId,
    candidate_lists::{CandidateList, CandidateListId},
    common::{DutchAddress, FullName},
    list_submitters::{ListSubmitter, ListSubmitterId},
    name_authorisations::{NameAuthorisation, NameAuthorisationId},
    persons::{Person, PersonId, PersonalData, Representative},
    political_groups::PoliticalGroup,
};

/// Domain events that mutate the application store.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum AppEvent {
    UpdatePoliticalGroup(PoliticalGroup),
    CreatePerson(Person),
    CreatePersonPersonalData {
        person_id: PersonId,
        name: FullName,
        personal_data: PersonalData,
    },
    UpdatePerson(Person),
    UpdatePersonPersonalData {
        person_id: PersonId,
        name: FullName,
        personal_data: PersonalData,
    },
    UpdatePersonAddress {
        person_id: PersonId,
        address: DutchAddress,
    },
    UpdatePersonRepresentative {
        person_id: PersonId,
        representative: Option<Representative>,
    },
    DeletePerson {
        person_id: PersonId,
    },
    CreateCandidateList(CandidateList),
    UpdateCandidateListDistricts {
        list_id: CandidateListId,
        electoral_districts: Vec<ElectoralDistrict>,
    },
    UpdateCandidateListOrder {
        list_id: CandidateListId,
        candidates: Vec<PersonId>,
    },
    AddCandidateToCandidateList {
        list_id: CandidateListId,
        person_id: PersonId,
    },
    RemoveCandidateFromCandidateList {
        list_id: CandidateListId,
        person_id: PersonId,
    },
    DeleteCandidateList(CandidateListId),

    CreateNameAuthorisation(NameAuthorisation),
    UpdateNameAuthorisation(NameAuthorisation),
    DeleteNameAuthorisation(NameAuthorisationId),

    UpdateListSubmitter(ListSubmitter),

    CreateSubstituteSubmitter(ListSubmitter),
    UpdateSubstituteSubmitter(ListSubmitter),
    DeleteSubstituteSubmitter {
        substitute_submitter_id: ListSubmitterId,
    },

    DeveloperLogin {
        stream_id: StreamId,
    },

    DownloadFile {
        file_name: String,
        download_path: String,
    },
    HideDownloadWarning,

    ExportCsv {
        file_name: String,
        file_size: usize,
        list_id: CandidateListId,
    },

    ImportCandidates {
        list_id: CandidateListId,
        file_name: String,
        file_size: usize,
        created_persons: Vec<Person>,
        updated_persons: Vec<Person>,
        candidates: Vec<PersonId>,
    },
}

impl Event for AppEvent {
    /// Return a stable category key for filtering in the audit log.
    fn category(&self) -> &'static str {
        match self {
            AppEvent::UpdatePoliticalGroup(_) => "political_group",
            AppEvent::CreatePerson(_)
            | AppEvent::CreatePersonPersonalData { .. }
            | AppEvent::UpdatePerson(_)
            | AppEvent::UpdatePersonPersonalData { .. }
            | AppEvent::UpdatePersonAddress { .. }
            | AppEvent::UpdatePersonRepresentative { .. }
            | AppEvent::DeletePerson { .. } => "person",
            AppEvent::CreateCandidateList(_)
            | AppEvent::UpdateCandidateListDistricts { .. }
            | AppEvent::UpdateCandidateListOrder { .. }
            | AppEvent::AddCandidateToCandidateList { .. }
            | AppEvent::RemoveCandidateFromCandidateList { .. }
            | AppEvent::DeleteCandidateList(_) => "candidate_list",
            AppEvent::CreateNameAuthorisation(_)
            | AppEvent::UpdateNameAuthorisation(_)
            | AppEvent::DeleteNameAuthorisation(_) => "name_authorisation",
            AppEvent::UpdateListSubmitter(_) => "list_submitter",
            AppEvent::CreateSubstituteSubmitter(_)
            | AppEvent::UpdateSubstituteSubmitter(_)
            | AppEvent::DeleteSubstituteSubmitter { .. } => "substitute_submitter",
            AppEvent::DeveloperLogin { .. }
            | AppEvent::DownloadFile { .. }
            | AppEvent::HideDownloadWarning
            | AppEvent::ExportCsv { .. }
            | AppEvent::ImportCandidates { .. } => "system",
        }
    }

    /// Return a stable snake_case key identifying the event variant.
    /// Variants that share a user-facing description share a key (e.g. both
    /// `CreatePerson` and `CreatePersonPersonalData` map to `create_person`).
    fn key(&self) -> &'static str {
        match self {
            AppEvent::UpdatePoliticalGroup(_) => "update_political_group",
            AppEvent::CreatePerson(_) | AppEvent::CreatePersonPersonalData { .. } => {
                "create_person"
            }
            AppEvent::UpdatePerson(_) | AppEvent::UpdatePersonPersonalData { .. } => {
                "update_person"
            }
            AppEvent::UpdatePersonAddress { .. } => "update_person_address",
            AppEvent::UpdatePersonRepresentative { .. } => "update_person_representative",
            AppEvent::DeletePerson { .. } => "delete_person",
            AppEvent::CreateCandidateList(_) => "create_candidate_list",
            AppEvent::UpdateCandidateListDistricts { .. } => "update_candidate_list_districts",
            AppEvent::UpdateCandidateListOrder { .. } => "update_candidate_list_order",
            AppEvent::AddCandidateToCandidateList { .. } => "add_candidate_to_list",
            AppEvent::RemoveCandidateFromCandidateList { .. } => "remove_candidate_from_list",
            AppEvent::DeleteCandidateList(_) => "delete_candidate_list",
            AppEvent::CreateNameAuthorisation(_) => "create_name_authorisation",
            AppEvent::UpdateNameAuthorisation(_) => "update_name_authorisation",
            AppEvent::DeleteNameAuthorisation(_) => "delete_name_authorisation",
            AppEvent::UpdateListSubmitter(_) => "update_list_submitter",
            AppEvent::CreateSubstituteSubmitter(_) => "create_substitute_submitter",
            AppEvent::UpdateSubstituteSubmitter(_) => "update_substitute_submitter",
            AppEvent::DeleteSubstituteSubmitter { .. } => "delete_substitute_submitter",
            AppEvent::DeveloperLogin { .. } => "developer_login",
            AppEvent::DownloadFile { .. } => "download_file",
            AppEvent::HideDownloadWarning => "hide_download_warning",
            AppEvent::ExportCsv { .. } => "export_csv",
            AppEvent::ImportCandidates { .. } => "import_csv",
        }
    }

    fn description(&self, locale: crate::Locale) -> String {
        super::event_info::event_description(self, locale)
    }

    fn details(&self) -> String {
        super::event_info::event_details(self)
    }
}
