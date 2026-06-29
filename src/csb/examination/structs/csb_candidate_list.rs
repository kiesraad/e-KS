use rand::{RngExt, rng};

use crate::candidate_lists::CandidateList;

pub struct CsbCandidateList {
    pub list: CandidateList,
    pub brp_error_count: usize,
}

impl CsbCandidateList {
    pub fn placeholder(candidate_list: CandidateList) -> Self {
        Self {
            list: candidate_list,
            brp_error_count: rng().random_range(0..=2),
        }
    }
}
