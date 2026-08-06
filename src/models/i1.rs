//! Model I 1: Proces-verbaal van het onderzoek naar de kandidatenlijsten.
//! This model is Dutch-only; unlike I 4 it only reports the omissions found
//! per list, it does not reproduce the candidate lists themselves. The
//! document text lives in the `templates/i1.md` Markdown template.

use textris_pdf::build::Textris;

use super::{
    Pdf,
    i4::{OmissionGroup, PublicSession},
    layout::markdown_document,
    markdown::model_template,
};
use crate::AppError;

#[derive(Debug)]
pub struct I1 {
    pub election_name: String,
    pub election_date: String,
    /// The session in which the central voting bureau examined the lists;
    /// same shape as the I 4 session.
    pub session: PublicSession,
    /// Empty when the examination found no omissions.
    pub found_omissions: Vec<OmissionGroup>,
}

model_template!(I1Template, I1, "models/templates/i1.md");

impl Pdf for I1 {
    fn document(&self) -> Result<Textris, AppError> {
        markdown_document(I1Template(self))
    }

    fn filename(&self) -> String {
        "i1-proces-verbaal.pdf".to_string()
    }
}
