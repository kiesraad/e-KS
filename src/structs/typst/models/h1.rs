use crate::{
    core::{ModelLocale, Pdf},
    finalise::DocumentData,
    list_designation::ListDesignation,
    typst::{TypstPerson, TypstPgModelData},
};
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct H1<'a> {
    #[serde(flatten)]
    common: &'a TypstPgModelData,
    previously_seated: bool,
    list_designation: ListDesignation,
    list_submitter: &'a TypstPerson,
    substitute_submitters: &'a Vec<TypstPerson>,
}

impl Pdf for H1<'_> {
    fn typst_template_name(&self) -> &'static str {
        "model-h1.typ"
    }

    fn filename(&self) -> String {
        match self.common.locale {
            ModelLocale::Nl => "h1-kandidatenlijst.pdf".to_string(),
            ModelLocale::Fry => "h1-kandidatelist.pdf".to_string(),
        }
    }
}

impl<'a> From<&'a DocumentData> for H1<'a> {
    fn from(data: &'a DocumentData) -> Self {
        Self {
            common: &data.model_data,
            previously_seated: data.previously_seated,
            list_designation: data.list_designation,
            list_submitter: &data.list_submitter,
            substitute_submitters: &data.substitute_submitters,
        }
    }
}
