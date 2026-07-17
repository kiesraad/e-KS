//! Model I 4: Proces-verbaal over geldigheid en nummering kandidatenlijsten.
//! This model is Dutch-only; the document text lives in the `templates/i4.md`
//! Markdown template.

use textris_pdf::build::Textris;

use super::{
    Pdf,
    layout::markdown_document,
    markdown::{filters, model_template},
};
use crate::AppError;

#[derive(Debug)]
pub struct I4 {
    pub election_name: String,
    pub election_date: String,
    pub public_session: PublicSession,
    pub found_omissions: Vec<OmissionGroup>,
    pub recovered_omissions: Vec<OmissionGroup>,
    pub invalid_lists: Vec<OmissionGroup>,
    pub removed_candidates: Vec<RemovedCandidates>,
    pub removed_designations: Vec<RemovedDesignation>,
    pub corrected_designations: Vec<CorrectedDesignation>,
    pub valid_lists: Vec<DistrictLists>,
    pub numbered_based_on_votes: Vec<NumberedOnVotes>,
    pub numbered_based_on_districts: Vec<NumberedOnDistricts>,
    /// `None`: room to write during the session; empty: no objections raised.
    pub objections: Option<Vec<String>>,
    pub response_objections: Option<String>,
}

#[derive(Debug)]
pub struct PublicSession {
    pub location: String,
    pub date: String,
    pub time: String,
    pub chair: String,
    pub members: Vec<String>,
}

/// Omissions for one list, identified by its designation and district(s).
#[derive(Debug)]
pub struct OmissionGroup {
    pub designation: String,
    pub electoral_district: String,
    pub omission_descriptions: Vec<String>,
}

#[derive(Debug)]
pub struct RemovedCandidates {
    pub designation: String,
    pub electoral_district: String,
    pub candidates: Vec<RemovedCandidate>,
}

#[derive(Debug)]
pub struct RemovedCandidate {
    pub name: String,
    pub reason: String,
}

#[derive(Debug)]
pub struct RemovedDesignation {
    pub designation: String,
    pub electoral_district: String,
    pub first_candidate_name: String,
    pub reason: String,
}

#[derive(Debug)]
pub struct CorrectedDesignation {
    pub first_candidate_name: String,
    pub electoral_district: String,
    pub submitted_designation: String,
    pub edited_designation: String,
}

#[derive(Debug)]
pub struct DistrictLists {
    pub electoral_district: String,
    pub lists: Vec<ValidList>,
}

#[derive(Debug)]
pub struct ValidList {
    pub designation: String,
    pub candidates: Vec<ValidListCandidate>,
}

#[derive(Debug)]
pub struct ValidListCandidate {
    pub last_name: String,
    pub initials: String,
    pub locality: String,
    pub position: usize,
}

#[derive(Debug)]
pub struct NumberedOnVotes {
    /// `None` when the number is still to be determined (rendered blank).
    pub position: Option<usize>,
    pub designation: String,
    pub previous_votes: u64,
}

#[derive(Debug)]
pub struct NumberedOnDistricts {
    /// `None` when the number is still to be determined (rendered blank).
    pub position: Option<usize>,
    pub designation: String,
    pub districts: u64,
}

model_template!(I4Template, I4, "models/templates/i4.md");

impl Pdf for I4 {
    fn document(&self) -> Result<Textris, AppError> {
        markdown_document(I4Template(self))
    }

    fn filename(&self) -> String {
        "i4-proces-verbaal.pdf".to_string()
    }
}
