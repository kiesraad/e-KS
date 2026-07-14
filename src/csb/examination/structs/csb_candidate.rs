use rand::{RngExt, rng};

use crate::persons::Person;

/// A candidate as shown on the CSB candidate list examination page: the
/// imported person, their position on the list and a placeholder count of BRP
/// errors until the real BRP checks are wired up.
pub struct CsbCandidate {
    pub person: Person,
    pub position: usize,
    pub brp_error_count: usize,
}

impl CsbCandidate {
    pub fn placeholder(person: Person, position: usize) -> Self {
        Self {
            person,
            position,
            brp_error_count: rng().random_range(0..=2),
        }
    }
}
