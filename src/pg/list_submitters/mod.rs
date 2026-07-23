//! List submitter management and related routes.
mod actions;
mod forms;
mod pages;

pub use crate::structs::list_submitters::{ListSubmitter, ListSubmitterData, ListSubmitterId};
pub use forms::ListSubmitterForm;
pub use pages::router;
