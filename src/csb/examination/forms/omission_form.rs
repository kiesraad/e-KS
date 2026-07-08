use serde::Deserialize;
use validate::Validate;

use crate::csb::Omission;

/// Form backing the "add omission" dialog. The category is not part of the form:
/// it is derived from the dialog's path parameters and set on the resulting
/// [`Omission`] by the handler after validation.
#[derive(Default, Deserialize, Debug, Validate)]
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
}

impl From<Omission> for OmissionForm {
    fn from(value: Omission) -> Self {
        OmissionForm {
            title: value.title,
            description: value.description,
            help_text: value.help_text,
        }
    }
}
