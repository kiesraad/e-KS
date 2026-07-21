//! Model H 1: Kandidatenlijst / Kandidatelist. The document text lives in the
//! `templates/h1.*.md` Markdown templates.

use textris_pdf::build::Textris;

use super::{
    Pdf,
    inputs::{ElectoralDistricts, ModelData, Person},
    layout::h_document,
    markdown::{filters, model_template},
};
use crate::{
    AppError,
    core::{ElectionType, ModelLocale},
    list_designation::ListDesignation,
};

#[derive(Debug)]
pub struct H1 {
    pub common: ModelData,
    pub electoral_districts: ElectoralDistricts,
    pub previously_seated: bool,
    pub list_designation: ListDesignation,
    pub list_submitter: Person,
    pub substitute_submitters: Vec<Person>,
}

model_template!(H1Nl, H1, "models/templates/h1.nl.md");
model_template!(H1Fy, H1, "models/templates/h1.fy.md");

impl Pdf for H1 {
    fn document(&self) -> Result<Textris, AppError> {
        match self.common.locale {
            ModelLocale::Nl => h_document(&self.common, H1Nl(self)),
            ModelLocale::Fry => h_document(&self.common, H1Fy(self)),
        }
    }

    fn filename(&self) -> String {
        match self.common.locale {
            ModelLocale::Nl => "h1-kandidatenlijst.pdf".to_string(),
            ModelLocale::Fry => "h1-kandidatelist.pdf".to_string(),
        }
    }
}

impl H1 {
    /// Whether the attachment checklist includes the declaration of intended
    /// residence: elections whose candidates must live in the area they
    /// represent.
    fn residence_declaration_required(&self) -> bool {
        matches!(
            self.common.election_type,
            ElectionType::Ps
                | ElectionType::Ws
                | ElectionType::Gr
                | ElectionType::Er
                | ElectionType::Kc
        )
    }
}
