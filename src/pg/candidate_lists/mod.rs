//! Candidate lists management.
//!
//! Scope: forms, extractors, pages, and structs that manage candidate lists,
//! their ordering, submitters, and related list summaries.
mod extractors;
mod forms;
pub(crate) mod importer;
mod pages;

pub use crate::structs::candidate_lists::{
    CandidateList, CandidateListId, CandidateListSummary, FullCandidateList,
};
pub use forms::{CandidateListCreateForm, CandidateListForm};
pub use pages::{CandidateListsPath, ViewCandidateListPath, router};
