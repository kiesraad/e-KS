use serde::{Deserialize, Serialize};

use crate::persons::PersonId;

#[derive(Default, Serialize, Deserialize, Clone, Debug)]
pub struct AddPerson {
    pub person_id: Option<PersonId>,
    pub remove_person_id: Option<PersonId>,
    pub added_position: Option<usize>,
}
