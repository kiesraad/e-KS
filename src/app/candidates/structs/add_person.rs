use serde::{Deserialize, Serialize};

use crate::{form::ValidationError, persons::PersonId};

#[derive(Default, Serialize, Deserialize, Clone, Debug)]
pub struct AddPerson {
    pub action: AddPersonAction,
    pub added_position: Option<usize>,
}

#[derive(Default, Serialize, Deserialize, PartialEq, Eq, Clone, Copy, Debug)]
pub enum AddPersonAction {
    #[default]
    None,
    AddAll,
    TogglePerson(PersonId),
}

impl std::str::FromStr for AddPersonAction {
    type Err = ValidationError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "" => Ok(AddPersonAction::None),
            "add-all" => Ok(AddPersonAction::AddAll),
            value => Ok(AddPersonAction::TogglePerson(value.parse()?)),
        }
    }
}

impl std::fmt::Display for AddPersonAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AddPersonAction::None => write!(f, ""),
            AddPersonAction::AddAll => write!(f, "add-all"),
            AddPersonAction::TogglePerson(person_id) => write!(f, "{}", person_id),
        }
    }
}
