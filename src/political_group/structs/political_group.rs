use crate::{
    id_newtype,
    political_group::structs::{
        authorized_agent::AuthorisedAgentId, list_submitter::ListSubmitterId,
    },
};
use chrono::{DateTime, Utc};

id_newtype!(pub struct PoliticalGroupId);

#[derive(Debug)]
pub struct PoliticalGroup {
    pub id: PoliticalGroupId,
    pub legal_name: String,
    pub display_name: String,

    pub authorised_agent_id: Option<AuthorisedAgentId>,
    pub list_submitter_id: Option<ListSubmitterId>,

    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
