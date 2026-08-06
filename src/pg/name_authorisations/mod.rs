//! Authorised agent management for a political group.
//!
//! Scope: forms, extractors, pages, and domain structs that create, update,
//! and remove authorised agents and expose the related routes.
mod actions;
mod extractors;
mod forms;
mod pages;

pub use forms::NameAuthorisationForm;
pub use pages::router;
