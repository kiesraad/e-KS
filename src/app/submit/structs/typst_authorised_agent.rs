use serde::Serialize;

use crate::{authorised_agents::AuthorisedAgent, common::Initials};

#[derive(Debug, Serialize)]
pub struct TypstAuthorisedAgent {
    pub last_name: String,
    pub initials: Initials,
}

impl From<&AuthorisedAgent> for TypstAuthorisedAgent {
    fn from(agent: &AuthorisedAgent) -> Self {
        TypstAuthorisedAgent {
            last_name: agent.name.last_name_with_prefix(),
            initials: agent.name.initials.clone(),
        }
    }
}
