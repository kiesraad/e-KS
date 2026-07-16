use serde::Serialize;

use crate::{
    core::Pdf,
    finalise::DocumentData,
    typst::{TypstDetailedCandidate, TypstPgModelData},
    utils::slugify_teletex,
};

#[derive(Debug, Serialize)]
pub struct H9<'a> {
    #[serde(flatten)]
    common: &'a TypstPgModelData,
    detailed_candidate: &'a TypstDetailedCandidate,
}

impl Pdf for H9<'_> {
    fn typst_template_name(&self) -> &'static str {
        "model-h9.typ"
    }

    fn filename(&self) -> String {
        format!(
            "h9-{}-{}.pdf",
            slugify_teletex(&self.detailed_candidate.candidate.last_name, true),
            self.detailed_candidate.candidate.position
        )
    }
}

impl<'a> From<(&'a DocumentData, &'a TypstDetailedCandidate)> for H9<'a> {
    fn from((data, candidate): (&'a DocumentData, &'a TypstDetailedCandidate)) -> Self {
        Self {
            common: &data.model_data,
            detailed_candidate: candidate,
        }
    }
}
