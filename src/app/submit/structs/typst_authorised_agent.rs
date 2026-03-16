use serde::Serialize;

use crate::authorised_agents::AuthorisedAgent;

#[derive(Debug, Serialize)]
pub struct TypstAuthorisedAgent {
    pub last_name: String,
    /// Initials as printed on the model, e.g., optionally including the first name
    pub initials: String,
}

impl From<&AuthorisedAgent> for TypstAuthorisedAgent {
    fn from(agent: &AuthorisedAgent) -> Self {
        TypstAuthorisedAgent {
            last_name: agent.name.last_name_with_prefix(),
            initials: agent.name.initials_with_first_name(),
        }
    }
}
