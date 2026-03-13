use serde::{Deserialize, Serialize};
use validate::Validate;

use crate::{
    OptionStringExt,
    common::{FirstName, FullName, Initials, LastName, LastNamePrefix},
};

#[derive(Default, Serialize, Deserialize, Clone, Debug, Validate)]
#[validate(target = "FullName")]
#[serde(default)]
pub struct FullNameForm {
    #[validate(parse = "FirstName", optional)]
    pub first_name: String,
    #[validate(parse = "LastName")]
    pub last_name: String,
    #[validate(parse = "LastNamePrefix", optional)]
    pub last_name_prefix: String,
    #[validate(parse = "Initials")]
    pub initials: String,
}

impl From<FullName> for FullNameForm {
    fn from(name: FullName) -> Self {
        FullNameForm {
            first_name: name.first_name.to_string_or_default(),
            last_name: name.last_name.to_string(),
            last_name_prefix: name.last_name_prefix.to_string_or_default(),
            initials: name.initials.to_string(),
        }
    }
}
