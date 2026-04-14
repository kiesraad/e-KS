//! List submitter management and related routes.
mod extractors;
mod forms;
mod pages;
mod structs;

pub use forms::ListSubmitterForm;
pub use pages::{ListSubmitterUpdatePath, router};
pub use structs::{ListSubmitter, ListSubmitterData, ListSubmitterId};
