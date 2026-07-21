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
/// optional ex-officio correction. The ex-officio layer takes precedence:
/// when present, all prior values are struck through and the ex-officio
/// value is shown in red.
pub struct PaperCorrected {
    pub imported: String,
    pub corrected: String,
    pub ex_officio: Option<String>,
}

impl PaperCorrected {
    pub fn new(imported: impl Into<String>, corrected: impl Into<String>) -> Self {
        Self {
            imported: imported.into(),
            corrected: corrected.into(),
            ex_officio: None,
        }
    }

    pub fn with_ex_officio(mut self, value: Option<String>) -> Self {
        self.ex_officio = value.filter(|v| v != &self.corrected);
        self
    }

    /// Pair one field of an imported entity with the same field of its
    /// corrected counterpart. A missing counterpart (deleted in the
    /// corrections) yields an empty corrected side; pass an absent imported
    /// side for entities added by the corrections.
    fn from_field<T>(
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
