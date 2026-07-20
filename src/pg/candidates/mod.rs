//! Candidate management within candidate lists.
//!
//! Scope: forms, extractors, pages, and structs for adding, updating,
//! reordering, and removing candidates.
mod extractors;
mod forms;
mod pages;

pub use crate::structs::candidates::{
    AddPerson, AddPersonAction, Candidate, CandidatePosition, CandidateWithProblems,
};
pub use forms::{AddPersonForm, CandidatePositionForm};
pub use pages::{AddCandidatePath, CreateCandidatePath, router};
