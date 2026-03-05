//! Validation and PDF generation for submission.
//!
//! Contains logic to validate application state and generate filled-in PDF documents.
mod pages;
mod structs;

pub use pages::{SubmitPath, router};
pub use structs::{h1::H1, h9::H9};
