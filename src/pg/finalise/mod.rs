//! The finalise page: validation of the application state and the download of
//! the filled-in PDF documents (built in `crate::models::documents`).
mod pages;
mod paths;
mod structs;

pub use pages::router;
pub use paths::FinalisePath;
pub use structs::problems::{AllProblems, EntityProblems};
