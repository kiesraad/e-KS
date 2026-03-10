use serde::{Deserialize, Serialize};
use validate::Validate;

use crate::{
    OptionStringExt,
    candidates::{AddPerson, AddPersonAction},
    form::TokenValue,
};

#[derive(Default, Serialize, Deserialize, Clone, Debug, Validate)]
#[validate(target = "AddPerson")]
pub struct AddPersonForm {
    #[validate(parse = "AddPersonAction")]
    #[serde(default)]
    pub action: String,
    #[validate(parse = "usize", optional)]
    pub added_position: String,
    #[validate(csrf)]
    pub csrf_token: TokenValue,
}

impl From<AddPerson> for AddPersonForm {
    fn from(add_person: AddPerson) -> Self {
        AddPersonForm {
            added_position: add_person.added_position.to_string_or_default(),
            action: add_person.action.to_string(),
            csrf_token: Default::default(),
        }
    }
}
