//! Validation and PDF generation for submission.
//!
//! Contains logic to validate application state and generate filled-in PDF documents.
mod pages;
mod structs;

pub use pages::{DownloadDocumentsPath, SubmitPath, documents, router};
pub use structs::{
    documents::{DocumentData, ZIP_CONTENT_TYPE},
    problems::{EntityProblems, GeneralProblems, ListProblems, PersonProblems, Problems},
};
