use serde::Serialize;

use crate::core::{Pdf, election::PublicSession};

#[derive(Debug, Default, Serialize)]
pub struct I4 {
    pub election_name: String,
    pub election_date: String,
    pub public_session: PublicSession,
    pub found_omissions: Vec<()>,
    pub recovered_omissions: Vec<()>,
    pub invalid_lists: Vec<()>,
    pub removed_candidates: Vec<()>,
    pub removed_designations: Vec<()>,
    pub corrected_designations: Vec<()>,
    pub valid_lists: Vec<()>,
    pub numbered_based_on_votes: Vec<()>,
    pub numbered_based_on_districts: Vec<()>,
    pub objections: Vec<String>,
    pub response_objections: Option<String>,
}

impl Pdf for I4 {
    fn typst_template_name(&self) -> &'static str {
        "model-i4.typ"
    }

    fn filename(&self) -> String {
        "i4-geldigheid-en-nummering.pdf".to_string()
    }
}
