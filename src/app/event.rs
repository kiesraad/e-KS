use serde::{Deserialize, Serialize};

use crate::{
    ElectoralDistrict,
    authorised_agents::{AuthorisedAgent, AuthorisedAgentId},
    candidate_lists::{CandidateList, CandidateListId},
    common::DutchAddress,
    list_submitters::{ListSubmitter, ListSubmitterId},
    persons::{Person, PersonId, Representative},
    political_groups::PoliticalGroup,
    substitute_list_submitters::{SubstituteSubmitter, SubstituteSubmitterId},
};

/// Domain events that mutate the application store.
#[derive(Debug, Serialize, Deserialize)]
pub enum AppEvent {
    UpdatePoliticalGroup(PoliticalGroup),
    CreatePerson(Person),
    UpdatePerson(Person),
    UpdatePersonAddress {
        person_id: PersonId,
        address: DutchAddress,
    },
    UpdatePersonRepresentative {
        person_id: PersonId,
        representative: Representative,
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
    UpdateCandidateListSubmitters {
        list_id: CandidateListId,
        list_submitter_id: Option<ListSubmitterId>,
        substitute_list_submitter_ids: Vec<SubstituteSubmitterId>,
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

    CreateAuthorisedAgent(AuthorisedAgent),
    UpdateAuthorisedAgent(AuthorisedAgent),
    DeleteAuthorisedAgent(AuthorisedAgentId),

    CreateListSubmitter(ListSubmitter),
    UpdateListSubmitter(ListSubmitter),
    DeleteListSubmitter {
        list_submitter_id: ListSubmitterId,
    },

    CreateSubstituteSubmitter(SubstituteSubmitter),
    UpdateSubstituteSubmitter(SubstituteSubmitter),
    DeleteSubstituteSubmitter {
        substitute_submitter_id: SubstituteSubmitterId,
    },
}
