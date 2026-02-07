use serde::{Deserialize, Serialize};

use crate::{
    authorised_agents::{AuthorisedAgent, AuthorisedAgentId},
    candidate_lists::{CandidateList, CandidateListId},
    list_submitters::{ListSubmitter, ListSubmitterId},
    persons::{Person, PersonId},
    political_groups::PoliticalGroup,
    substitute_list_submitters::{SubstituteSubmitter, SubstituteSubmitterId},
};

#[derive(Debug, Serialize, Deserialize)]
pub enum AppEvent {
    UpdatePoliticalGroup(PoliticalGroup),

    CreatePerson(Person),
    UpdatePerson(Person),
    DeletePerson(PersonId),

    CreateCandidateList(CandidateList),
    UpdateCandidateList(CandidateList),
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
