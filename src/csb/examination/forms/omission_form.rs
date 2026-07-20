use serde::Deserialize;
use validate::Validate;

use crate::{ElectoralDistrict, candidate_lists::CandidateListId, csb::Omission};

/// Form backing the "add omission" dialog. The category is not part of the form:
/// it is derived from the dialog's path parameters and set on the resulting
/// [`Omission`] by the handler after validation.
#[derive(Deserialize, Debug, Validate)]
#[validate(target = "Omission")]
#[serde(default)]
pub struct OmissionForm {
    #[validate(not_empty)]
    pub title: String,
    /// The description shown on model I 1.
    #[validate(not_empty)]
    pub description: String,
    /// The note added to the omission letter ("verzuimbrief").
    pub help_text: String,
    /// Whether the omission is recoverable ("herstelbaar"). Rendered as a
    /// checkbox: when it is unchecked the browser submits nothing, so serde
    /// falls back to `false` here, explicitly marking the omission irreparable.
    #[serde(default)]
    pub recoverable: bool,
    /// The electoral districts selected for a CandidateList omission.
    /// Ignored for other omission types; validated in the handler.
    #[validate(ignore)]
    pub electoral_districts: Vec<ElectoralDistrict>,
    /// The candidate lists selected for a Candidate omission.
    /// Ignored for other omission types; validated in the handler.
    #[validate(ignore)]
    pub candidate_lists: Vec<CandidateListId>,
}

impl Default for OmissionForm {
    fn default() -> Self {
        OmissionForm {
            title: String::new(),
            description: String::new(),
            help_text: String::new(),
            // A fresh omission is recoverable unless the committee marks it
            // otherwise (the common case); presets override this via the dialog.
            recoverable: true,
            electoral_districts: Vec::new(),
            candidate_lists: Vec::new(),
        }
    }
}

impl From<Omission> for OmissionForm {
    fn from(value: Omission) -> Self {
        OmissionForm {
            title: value.title,
            description: value.description,
            help_text: value.help_text,
            recoverable: value.recoverable,
            electoral_districts: Vec::new(),
            candidate_lists: Vec::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unchecked_recoverable_checkbox_submits_as_false() {
        // An unchecked checkbox is omitted from the submitted body, which must
        // mark the omission irreparable rather than falling back to the form's
        // recoverable-by-default value.
        let form: OmissionForm =
            serde_urlencoded::from_str("title=t&description=d&help_text=").unwrap();
        assert!(!form.recoverable);
    }

    #[test]
    fn checked_recoverable_checkbox_submits_as_true() {
        let form: OmissionForm =
            serde_urlencoded::from_str("title=t&description=d&recoverable=true").unwrap();
        assert!(form.recoverable);
    }

    #[test]
    fn fresh_form_defaults_to_recoverable() {
        assert!(OmissionForm::default().recoverable);
    }
}
