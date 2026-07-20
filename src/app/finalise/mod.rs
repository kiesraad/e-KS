//! Validation and PDF generation for submission.
//!
//! Contains logic to validate application state and generate filled-in PDF documents.
mod pages;
mod structs;

pub use pages::{FinalisePath, router};
pub use structs::{
    documents::DocumentData,
    problems::{AllProblems, EntityProblems},
};
