use crate::{
    core::{ModelLocale, Pdf},
    submit::{
        DocumentData,
        structs::{
            TypstPerson, typst_model_data::TypstModelData,
            typst_name_authorisation::TypstNameAuthorisation,
        },
    },
};
use serde::Serialize;

/// Either an H3-1 or H3-2 model, depending on the number of name authorisations provided.
#[derive(Debug, Serialize)]
pub struct H3<'a> {
    #[serde(flatten)]
    common: &'a TypstModelData,
    list_submitter: &'a TypstPerson,
    name_authorisations: &'a Vec<TypstNameAuthorisation>,
}

impl H3<'_> {
    fn use_h3_2(&self) -> bool {
        self.name_authorisations.len() > 1
    }
}

impl Pdf for H3<'_> {
    fn typst_template_name(&self) -> &'static str {
        if self.use_h3_2() {
            "model-h3-2.typ"
        } else {
            "model-h3-1.typ"
        }
    }

    fn filename(&self) -> String {
        match (self.common.locale, self.use_h3_2()) {
            (ModelLocale::Nl, false) => "h3-1-aanduiding.pdf",
            (ModelLocale::Fry, false) => "h3-1-oantsjutting.pdf",
            (ModelLocale::Nl, true) => "h3-2-samengevoegde-aanduiding.pdf",
            (ModelLocale::Fry, true) => "h3-2-gearfoege-oantsjutting.pdf",
        }
        .to_string()
    }
}

impl<'a> From<&'a DocumentData> for H3<'a> {
    fn from(data: &'a DocumentData) -> Self {
        Self {
            common: &data.model_data,
            list_submitter: &data.list_submitter,
            name_authorisations: &data.name_authorisations,
        }
    }
}
