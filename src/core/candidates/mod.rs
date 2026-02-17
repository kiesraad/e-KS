mod extractors;
mod pages;
mod structs;

pub use pages::{AddCandidatePath, CreateCandidatePath, router};
pub use structs::{Candidate, CandidatePosition, CandidatePositionForm};
