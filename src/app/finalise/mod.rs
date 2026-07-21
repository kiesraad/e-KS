//! The finalise page: validation of the application state and the download of
//! the filled-in PDF documents (built in `crate::models::documents`).
mod pages;
mod structs;

pub use pages::{FinalisePath, router};
pub use structs::problems::{AllProblems, EntityProblems};
