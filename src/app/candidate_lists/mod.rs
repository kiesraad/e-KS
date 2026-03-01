//! Candidate lists management.
//!
//! Scope: forms, extractors, pages, and structs that manage candidate lists,
//! their ordering, submitters, and related list summaries.
mod extractors;
mod forms;
mod pages;
mod structs;

pub use forms::{CandidateListCreateForm, CandidateListForm};
pub use pages::router;
pub use structs::{
    CandidateList, CandidateListId, CandidateListSummary, FullCandidateList, ListSubmitterForm,
};
