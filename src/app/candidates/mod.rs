mod extractors;
mod forms;
mod pages;
mod structs;

pub use forms::CandidatePositionForm;
pub use pages::{AddCandidatePath, CreateCandidatePath, router};
pub use structs::{Candidate, CandidatePosition};
