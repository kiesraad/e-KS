use serde::Serialize;

use crate::name_authorisations::NameAuthorisation;

#[derive(Debug, Default, Serialize)]
pub struct TypstNameAuthorisation {
    pub last_name: String,
    /// Initials as printed on the model, e.g., optionally including the first name
    pub initials: String,
    pub legal_name: String,
}

impl From<&NameAuthorisation> for TypstNameAuthorisation {
    fn from(name_authorisation: &NameAuthorisation) -> Self {
        TypstNameAuthorisation {
            last_name: name_authorisation.name.last_name_with_prefix(),
            initials: name_authorisation.name.initials_with_first_name(),
            legal_name: name_authorisation.legal_name.to_string(),
        }
    }
}
