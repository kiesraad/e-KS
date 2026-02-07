use serde::{Deserialize, Serialize};

use crate::{
    DutchAddress, ElectoralDistrict, FullName, UtcDateTime,
    authorised_agents::{AuthorisedAgent, AuthorisedAgentId},
    candidate_lists::{CandidateList, CandidateListId},
    list_submitters::{ListSubmitter, ListSubmitterId},
    persons::{Person, PersonId, PersonalInfo},
    political_groups::PoliticalGroup,
    substitute_list_submitters::{SubstituteSubmitter, SubstituteSubmitterId},
};

#[derive(Debug, Serialize, Deserialize)]
pub enum AppEvent {
    UpdatePoliticalGroup(PoliticalGroup),

    CreatePerson(Person),
    UpdatePerson(Person),
    UpdatePersonPersonalInfo(PersonalInfo),
    UpdatePersonAddress {
        person_id: PersonId,
        address: DutchAddress,
        updated_at: UtcDateTime,
    },
    UpdatePersonRepresentative {
        person_id: PersonId,
        representative: FullName,
        address: DutchAddress,
        updated_at: UtcDateTime,
    },
    DeletePerson(PersonId),

    CreateCandidateList(CandidateList),
    UpdateCandidateList(CandidateList),
    UpdateCandidateListDistricts {
        list_id: CandidateListId,
        electoral_districts: Vec<ElectoralDistrict>,
        updated_at: UtcDateTime,
    },
    UpdateCandidateListOrder {
        list_id: CandidateListId,
        candidates: Vec<PersonId>,
        updated_at: UtcDateTime,
    },
    UpdateCandidateListSubmitters {
        list_id: CandidateListId,
        list_submitter_id: Option<ListSubmitterId>,
        substitute_list_submitter_ids: Vec<SubstituteSubmitterId>,
        updated_at: UtcDateTime,
    },
    AddCandidateToCandidateList {
        list_id: CandidateListId,
        person_id: PersonId,
        updated_at: UtcDateTime,
    },
    RemoveCandidateFromCandidateList {
        list_id: CandidateListId,
        person_id: PersonId,
        updated_at: UtcDateTime,
    },
    RemoveCandidateFromAllCandidateLists {
        person_id: PersonId,
        updated_at: UtcDateTime,
    },
    DeleteCandidateList(CandidateListId),

    CreateAuthorisedAgent(AuthorisedAgent),
    UpdateAuthorisedAgent(AuthorisedAgent),
    DeleteAuthorisedAgent(AuthorisedAgentId),

    CreateListSubmitter(ListSubmitter),
    UpdateListSubmitter(ListSubmitter),
    DeleteListSubmitter(ListSubmitterId),

    CreateSubstituteSubmitter(SubstituteSubmitter),
    UpdateSubstituteSubmitter(SubstituteSubmitter),
    DeleteSubstituteSubmitter(SubstituteSubmitterId),
}
