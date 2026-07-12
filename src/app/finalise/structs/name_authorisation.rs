use crate::{
    models::inputs::NameAuthorisation as ModelNameAuthorisation,
    name_authorisations::NameAuthorisation,
};

impl From<&NameAuthorisation> for ModelNameAuthorisation {
    fn from(name_authorisation: &NameAuthorisation) -> Self {
        ModelNameAuthorisation {
            last_name: name_authorisation.name.last_name_with_prefix(),
            initials: name_authorisation.name.initials_with_first_name(),
            legal_name: name_authorisation.legal_name.to_string(),
        }
    }
}
