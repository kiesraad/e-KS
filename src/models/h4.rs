//! Model H 4: Ondersteuningsverklaring / Stipeferklearring. The document text
//! lives in the `templates/h4.*.md` Markdown templates.

use textris_pdf::build::Textris;

use super::{
    Pdf,
    inputs::ModelData,
    layout::h_document,
    markdown::{filters, model_template},
};
use crate::{
    AppError,
    core::{ElectionType, ModelLocale},
};

#[derive(Debug)]
pub struct H4 {
    pub common: ModelData,
}

model_template!(H4Nl, H4, "models/templates/h4.nl.md");
model_template!(H4Fy, H4, "models/templates/h4.fy.md");

impl Pdf for H4 {
    fn document(&self) -> Result<Textris, AppError> {
        match self.common.locale {
            ModelLocale::Nl => h_document(&self.common, H4Nl(self)),
            ModelLocale::Fry => h_document(&self.common, H4Fy(self)),
        }
    }

    fn filename(&self) -> String {
        match self.common.locale {
            ModelLocale::Nl => "h4-ondersteuningsverklaring.pdf".to_string(),
            ModelLocale::Fry => "h4-stipeferklearring.pdf".to_string(),
        }
    }
}

impl H4 {
    /// Word choice depending on whether the voter register is kept by a
    /// municipality (`gr`) or a public body (`non_gr`).
    fn municipality(&self, gr: &str, non_gr: &str) -> String {
        match self.common.election_type {
            ElectionType::Er => non_gr.to_string(),
            ElectionType::Tk => format!("{gr} / {non_gr}"),
            _ => gr.to_string(),
        }
    }
}
