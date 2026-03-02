//! Substitute list submitter management and related routes.
mod extractors;
mod forms;
mod pages;
mod structs;

pub use forms::SubstituteSubmitterForm;
pub use pages::router;
pub use structs::{SubstituteSubmitter, SubstituteSubmitterId};
