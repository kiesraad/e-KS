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
pub enum PgEvent {
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

impl Event for PgEvent {
    /// Return a stable category key for filtering in the audit log.
    fn category(&self) -> &'static str {
        match self {
            PgEvent::UpdatePoliticalGroup(_) => "political_group",
            PgEvent::CreatePerson(_)
            | PgEvent::CreatePersonPersonalData { .. }
            | PgEvent::UpdatePerson(_)
            | PgEvent::UpdatePersonPersonalData { .. }
            | PgEvent::UpdatePersonAddress { .. }
            | PgEvent::UpdatePersonRepresentative { .. }
            | PgEvent::DeletePerson { .. } => "person",
            PgEvent::CreateCandidateList(_)
            | PgEvent::UpdateCandidateListDistricts { .. }
            | PgEvent::UpdateCandidateListOrder { .. }
            | PgEvent::AddCandidateToCandidateList { .. }
            | PgEvent::RemoveCandidateFromCandidateList { .. }
            | PgEvent::DeleteCandidateList(_) => "candidate_list",
            PgEvent::CreateNameAuthorisation(_)
            | PgEvent::UpdateNameAuthorisation(_)
            | PgEvent::DeleteNameAuthorisation(_) => "name_authorisation",
            PgEvent::UpdateListSubmitter(_) => "list_submitter",
            PgEvent::CreateSubstituteSubmitter(_)
            | PgEvent::UpdateSubstituteSubmitter(_)
            | PgEvent::DeleteSubstituteSubmitter { .. } => "substitute_submitter",
            PgEvent::DeveloperLogin { .. }
            | PgEvent::DownloadFile { .. }
            | PgEvent::HideDownloadWarning
            | PgEvent::ExportCsv { .. }
            | PgEvent::ImportCandidates { .. } => "system",
        }
    }

    /// Return a stable snake_case key identifying the event variant.
    /// Variants that share a user-facing description share a key (e.g. both
    /// `CreatePerson` and `CreatePersonPersonalData` map to `create_person`).
    fn key(&self) -> &'static str {
        match self {
            PgEvent::UpdatePoliticalGroup(_) => "update_political_group",
            PgEvent::CreatePerson(_) | PgEvent::CreatePersonPersonalData { .. } => "create_person",
            PgEvent::UpdatePerson(_) | PgEvent::UpdatePersonPersonalData { .. } => "update_person",
            PgEvent::UpdatePersonAddress { .. } => "update_person_address",
            PgEvent::UpdatePersonRepresentative { .. } => "update_person_representative",
            PgEvent::DeletePerson { .. } => "delete_person",
            PgEvent::CreateCandidateList(_) => "create_candidate_list",
            PgEvent::UpdateCandidateListDistricts { .. } => "update_candidate_list_districts",
            PgEvent::UpdateCandidateListOrder { .. } => "update_candidate_list_order",
            PgEvent::AddCandidateToCandidateList { .. } => "add_candidate_to_list",
            PgEvent::RemoveCandidateFromCandidateList { .. } => "remove_candidate_from_list",
            PgEvent::DeleteCandidateList(_) => "delete_candidate_list",
            PgEvent::CreateNameAuthorisation(_) => "create_name_authorisation",
            PgEvent::UpdateNameAuthorisation(_) => "update_name_authorisation",
            PgEvent::DeleteNameAuthorisation(_) => "delete_name_authorisation",
            PgEvent::UpdateListSubmitter(_) => "update_list_submitter",
            PgEvent::CreateSubstituteSubmitter(_) => "create_substitute_submitter",
            PgEvent::UpdateSubstituteSubmitter(_) => "update_substitute_submitter",
            PgEvent::DeleteSubstituteSubmitter { .. } => "delete_substitute_submitter",
            PgEvent::DeveloperLogin { .. } => "developer_login",
            PgEvent::DownloadFile { .. } => "download_file",
            PgEvent::HideDownloadWarning => "hide_download_warning",
            PgEvent::ExportCsv { .. } => "export_csv",
            PgEvent::ImportCandidates { .. } => "import_csv",
        }
    }

    fn description(&self, locale: crate::Locale) -> String {
        super::event_info::event_description(self, locale)
    }

    fn details(&self) -> String {
        super::event_info::event_details(self)
    }
}
