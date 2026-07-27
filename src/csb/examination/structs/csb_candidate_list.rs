use crate::structs::candidate_lists::CandidateList;

pub struct CsbCandidateList {
    pub list: CandidateList,
    pub brp_error_count: usize,
    pub is_paper_added: bool,
}
