use serde::Deserialize;

use crate::candidate_lists::structs::CandidateRecord;

#[derive(Deserialize)]
pub struct ListImportPayload {
    pub records: Vec<CandidateRecord>,
}
