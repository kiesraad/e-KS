mod correction_field;
mod csb_candidate;
mod csb_candidate_list;
mod paper_corrected;

pub use correction_field::CandidateCorrectionField;
pub use csb_candidate::CsbCandidate;
pub use csb_candidate_list::CsbCandidateList;
pub use paper_corrected::{
    PaperCorrected, PaperCorrectedNameAuthorisation, PaperCorrectedPersonDetails,
    PaperCorrectedPoliticalGroupInfo, PaperCorrectedSubmitter, paper_corrected_list_submitter,
    paper_corrected_name_authorisations, paper_corrected_substitute_submitters,
};
