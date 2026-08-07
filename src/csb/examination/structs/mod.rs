mod all_csb_corrections;
mod all_omissions;
mod correction_field;
mod csb_candidate;
mod csb_candidate_list;
mod paper_corrected;

pub use all_csb_corrections::AllCsbCorrections;
pub use all_omissions::AllOmissions;
pub use correction_field::CandidateCorrectionField;
pub use csb_candidate::CsbCandidate;
pub use csb_candidate_list::CsbCandidateList;
pub use paper_corrected::{
    PaperCorrected, PaperCorrectedNameAuthorisation, PaperCorrectedPersonDetails,
    PaperCorrectedPoliticalGroupInfo, PaperCorrectedSubmitter, paper_corrected_list_submitter,
    paper_corrected_name_authorisations, paper_corrected_substitute_submitters,
};

pub struct RestorationStatus {
    pub has_omissions: bool,
    pub has_corrections: bool,
}
