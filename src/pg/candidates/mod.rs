//! Candidate management within candidate lists.
//!
//! Scope: forms, extractors, pages, and structs for adding, updating,
//! reordering, and removing candidates.
mod extractors;
mod forms;
mod pages;
mod paths;

pub use forms::{AddPersonForm, CandidatePositionForm};
pub use pages::router;
pub use paths::{AddCandidatePath, CreateCandidatePath};
