//! View structs pairing imported values with their paper-corrected
//! counterparts, rendered by the `paper_corrected.html` template macro.

mod name_authorisation;
mod person_details;
mod political_group_info;
mod submitter;

pub use name_authorisation::{
    PaperCorrectedNameAuthorisation, paper_corrected_name_authorisations,
};
pub use person_details::PaperCorrectedPersonDetails;
pub use political_group_info::PaperCorrectedPoliticalGroupInfo;
pub use submitter::{
    PaperCorrectedSubmitter, paper_corrected_list_submitter, paper_corrected_substitute_submitters,
};

/// An imported value paired with its paper-corrected counterpart and an
/// optional CSB correction. The CSB corrected value takes precedence:
/// when present, all prior values are struck through and the CSB corrected
/// value is shown in red.
pub struct PaperCorrected {
    pub imported: String,
    pub corrected: String,
    pub csb_corrected: Option<String>,
}

impl PaperCorrected {
    pub fn new(imported: impl Into<String>, corrected: impl Into<String>) -> Self {
        Self {
            imported: imported.into(),
            corrected: corrected.into(),
            csb_corrected: None,
        }
    }

    pub fn with_csb_correction(mut self, value: Option<String>) -> Self {
        self.csb_corrected = value.filter(|v| v != &self.corrected);
        self
    }

    /// Pair one field of an imported entity with the same field of its
    /// corrected counterpart. A missing counterpart (deleted in the
    /// corrections) yields an empty corrected side; pass an absent imported
    /// side for entities added by the corrections.
    pub fn from_field<T>(
        imported: Option<&T>,
        corrected: Option<&T>,
        field: impl Fn(&T) -> String,
    ) -> Self {
        Self::new(
            imported.map(&field).unwrap_or_default(),
            corrected.map(&field).unwrap_or_default(),
        )
    }

    pub fn differs(&self) -> bool {
        self.imported != self.corrected
    }
}
