use serde::Serialize;

use crate::persons::Person;

#[derive(Debug, Clone, Serialize)]
pub struct Candidate {
    pub position: i32,
    pub person: Person,
}
