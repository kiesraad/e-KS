use serde::Serialize;

use crate::name_authorisations::NameAuthorisation;

#[derive(Debug, Serialize)]
pub struct TypstAuthorisedAgent {
    pub last_name: String,
    /// Initials as printed on the model, e.g., optionally including the first name
    pub initials: String,
}

impl From<&NameAuthorisation> for TypstAuthorisedAgent {
    fn from(agent: &NameAuthorisation) -> Self {
        TypstAuthorisedAgent {
            last_name: agent.name.last_name_with_prefix(),
            initials: agent.name.initials_with_first_name(),
        }
    }
}
