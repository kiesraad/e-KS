use serde::Serialize;
use uuid::Uuid;

use crate::persons::Person;

#[derive(Debug, Clone, Serialize)]
pub struct Candidate {
    pub list_id: Uuid,
    pub position: i32,
    pub person: Person,
}
