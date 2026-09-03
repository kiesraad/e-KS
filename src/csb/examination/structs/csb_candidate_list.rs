use crate::{
    csb::examination::structs::{BrpCheckState, RestorationStatus},
    structs::candidate_lists::CandidateList,
};

pub struct CsbCandidateList {
    pub list: CandidateList,
    pub brp: BrpCheckState,
    pub restoration_status: RestorationStatus,
    pub is_paper_added: bool,
}
