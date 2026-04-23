use serde::{Deserialize, Serialize};
use validate::Validate;

use crate::{
    OptionStringExt,
    common::{FullName, Initials, LastName, LastNamePrefix},
};

/// Name form for list submitters and substitute submitters.
///
/// Unlike [`FullNameForm`][crate::common::FullNameForm], submitters don't
/// collect a first name — the data model still carries the field (reused
/// `FullName` struct), but the UI never presents it, so it stays `None`.
#[derive(Default, Serialize, Deserialize, Clone, Debug, Validate)]
#[validate(target = "FullName")]
#[serde(default)]
pub struct SubmitterNameForm {
    #[validate(parse = "LastName")]
    pub last_name: String,
    #[validate(parse = "LastNamePrefix", optional)]
    pub last_name_prefix: String,
    #[validate(parse = "Initials")]
    pub initials: String,
}

impl From<FullName> for SubmitterNameForm {
    fn from(name: FullName) -> Self {
        SubmitterNameForm {
            last_name: name.last_name.to_string(),
            last_name_prefix: name.last_name_prefix.to_string_or_default(),
            initials: name.initials.to_string(),
        }
    }
}
