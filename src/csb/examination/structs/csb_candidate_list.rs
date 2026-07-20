use rand::{RngExt, rng};

use crate::candidate_lists::CandidateList;

pub struct CsbCandidateList {
    pub list: CandidateList,
    pub brp_error_count: usize,
    pub is_paper_added: bool,
}

impl CsbCandidateList {
    pub fn placeholder(candidate_list: CandidateList) -> Self {
        Self {
            list: candidate_list,
            brp_error_count: rng().random_range(0..=2),
            is_paper_added: false,
        }
    }

    /// A list only present in the paper-corrected projection
    pub fn paper_added(candidate_list: CandidateList) -> Self {
        Self {
            list: candidate_list,
            // TODO: This is a placeholder value, the real value should be calculated based on the candidate list data.
            brp_error_count: rng().random_range(0..=2),
            is_paper_added: true,
        }
    }
}
