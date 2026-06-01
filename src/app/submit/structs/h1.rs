use crate::{
    core::{ModelLocale, Pdf},
    submit::{
        DocumentData,
        structs::{TypstPerson, typst_model_data::TypstModelData},
    },
};
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct H1<'a> {
    #[serde(flatten)]
    common: &'a TypstModelData,
    previously_seated: bool,
    list_submitter: &'a TypstPerson,
    substitute_submitters: &'a Vec<TypstPerson>,
    nr_of_name_authorisations: usize,
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
            list_submitter: &data.list_submitter,
            substitute_submitters: &data.substitute_submitters,
            nr_of_name_authorisations: data.name_authorisations.len(),
        }
    }
}
