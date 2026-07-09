use serde::Serialize;

use crate::{
    core::{ModelLocale, Pdf},
    finalise::DocumentData,
    typst::TypstPgModelData,
};

#[derive(Debug, Serialize)]
pub struct H4<'a> {
    #[serde(flatten)]
    common: &'a TypstPgModelData,
}

impl Pdf for H4<'_> {
    fn typst_template_name(&self) -> &'static str {
        "model-h4.typ"
    }

    fn filename(&self) -> String {
        match self.common.locale {
            ModelLocale::Nl => "h4-ondersteuningsverklaring.pdf".to_string(),
            ModelLocale::Fry => "h4-stipeferklearring.pdf".to_string(),
        }
    }
}

impl<'a> From<&'a DocumentData> for H4<'a> {
    fn from(data: &'a DocumentData) -> Self {
        Self {
            common: &data.model_data,
        }
    }
}
