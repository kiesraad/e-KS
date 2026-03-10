use serde::{Deserialize, Serialize};
use validate::Validate;

use crate::{
    OptionStringExt,
    candidates::{AddPerson, AddPersonAction},
    form::TokenValue,
    persons::PersonId,
};

#[derive(Default, Serialize, Deserialize, Clone, Debug, Validate)]
#[validate(target = "AddPerson")]
pub struct AddPersonForm {
    #[validate(parse = "PersonId", optional)]
    #[serde(default)]
    pub person_id: String,
    #[validate(parse = "PersonId", optional)]
    #[serde(default)]
    pub remove_person_id: String,
    #[validate(parse = "AddPersonAction", optional)]
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
            person_id: add_person.person_id.to_string_or_default(),
            remove_person_id: add_person.remove_person_id.to_string_or_default(),
            added_position: add_person.added_position.to_string_or_default(),
            action: add_person.action.to_string_or_default(),
            csrf_token: Default::default(),
        }
    }
}
