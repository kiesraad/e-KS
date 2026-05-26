//! Authorised agent management for a political group.
//!
//! Scope: forms, extractors, pages, and domain structs that create, update,
//! and remove authorised agents and expose the related routes.
mod extractors;
mod forms;
mod pages;
mod structs;

pub use forms::NameAuthorisationForm;
pub use pages::{NameAuthorisationUpdatePath, router};
pub use structs::{NameAuthorisation, NameAuthorisationId};
