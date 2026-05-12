//! Validation and PDF generation for submission.
//!
//! Contains logic to validate application state and generate filled-in PDF documents.
mod pages;
mod structs;

pub use pages::{SubmitPath, router};
pub use structs::{
    documents::DocumentData,
    potential_problems::{
        GeneralProblems, ListProblems, PersonProblems, PotentialProblems, Problematic, Problems,
        Severity,
    },
};
