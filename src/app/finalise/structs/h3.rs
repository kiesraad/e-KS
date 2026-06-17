use crate::{
    core::{ModelLocale, Pdf},
    finalise::{
        DocumentData,
        structs::{
            TypstPerson, typst_model_data::TypstModelData,
            typst_name_authorisation::TypstNameAuthorisation,
        },
    },
    list_designation::ListDesignation,
};
use serde::Serialize;

/// Either an H3-1 or H3-2 model, depending on the list designation.
#[derive(Debug, Serialize)]
pub struct H3<'a> {
    #[serde(flatten)]
    common: &'a TypstModelData,
    list_designation: ListDesignation,
    list_submitter: &'a TypstPerson,
    name_authorisations: &'a Vec<TypstNameAuthorisation>,
}

impl Pdf for H3<'_> {
    fn typst_template_name(&self) -> &'static str {
        if self.list_designation == ListDesignation::Combined {
            "model-h3-2.typ"
        } else {
            "model-h3-1.typ"
        }
    }

    fn filename(&self) -> String {
        match (self.common.locale, self.list_designation) {
            (ModelLocale::Nl, ListDesignation::Combined) => "h3-2-samengevoegde-aanduiding.pdf",
            (ModelLocale::Fry, ListDesignation::Combined) => "h3-2-gearfoege-oantsjutting.pdf",
            (ModelLocale::Nl, _) => "h3-1-aanduiding.pdf",
            (ModelLocale::Fry, _) => "h3-1-oantsjutting.pdf",
        }
        .to_string()
    }
}

impl<'a> From<&'a DocumentData> for H3<'a> {
    fn from(data: &'a DocumentData) -> Self {
        Self {
            common: &data.model_data,
            list_designation: data.list_designation,
            list_submitter: &data.list_submitter,
            name_authorisations: &data.name_authorisations,
        }
    }
}
