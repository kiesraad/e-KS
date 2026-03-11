//! Candidate management within candidate lists.
//!
//! Scope: forms, extractors, pages, and structs for adding, updating,
//! reordering, and removing candidates.
mod extractors;
mod forms;
mod pages;
mod structs;

pub use forms::{AddPersonForm, CandidatePositionForm};
pub use pages::{AddCandidatePath, CreateCandidatePath, router};
pub use structs::{AddPerson, AddPersonAction, Candidate, CandidatePosition};
