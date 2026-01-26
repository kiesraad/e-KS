use crate::{
    id_newtype,
    political_groups::{AuthorisedAgentId, ListSubmitterId},
};
use chrono::{DateTime, Utc};

id_newtype!(pub struct PoliticalGroupId);

#[derive(Default, Debug, Clone)]
pub struct PoliticalGroup {
    pub id: PoliticalGroupId,

    pub long_list_allowed: Option<bool>,

    pub legal_name: String,
    pub legal_name_confirmed: Option<bool>,

    pub display_name: String,
    pub display_name_confirmed: Option<bool>,

    pub authorised_agent_id: Option<AuthorisedAgentId>,
    pub list_submitter_id: Option<ListSubmitterId>,

    #[allow(unused)]
    pub created_at: DateTime<Utc>,

    #[allow(unused)]
    pub updated_at: DateTime<Utc>,
}
