use crate::{
    csb::examination::structs::RestorationStatus, structs::candidate_lists::CandidateList,
};

pub struct CsbCandidateList {
    pub list: CandidateList,
    pub brp_error_count: usize,
    pub restoration_status: RestorationStatus,
    pub is_paper_added: bool,
}
