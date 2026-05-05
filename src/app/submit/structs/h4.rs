use serde::Serialize;

use crate::{
    core::{ElectionType, ModelLocale, Pdf},
    submit::{
        DocumentData,
        structs::{typst_candidate::TypstCandidate, typst_datetime::TypstDatetime},
    },
};

#[derive(Debug, Serialize)]
pub struct H4<'a> {
    election_name: String,
    election_type: ElectionType,
    designation: &'a str,
    candidates: &'a Vec<TypstCandidate>,
    timestamp: &'a TypstDatetime,
    locale: ModelLocale,
    filename: &'static str,
}

impl Pdf for H4<'_> {
    fn typst_template_name(&self) -> &'static str {
        "model-h4.typ"
    }

    fn filename(&self) -> &str {
        self.filename
    }
}

impl<'a> From<&'a DocumentData> for H4<'a> {
    fn from(data: &'a DocumentData) -> Self {
        let locale = data.locale;
        let election = data.election;

        let filename = match locale {
            ModelLocale::Nl => "h4-ondersteuningsverklaring.pdf",
            ModelLocale::Fry => "h4-stipeferklearring.pdf",
        };

        Self {
            election_name: election.formal_title(locale),
            election_type: election.election_type(),
            designation: &data.designation,
            candidates: &data.ordered_candidates,
            timestamp: &data.timestamp,
            locale,
            filename,
        }
    }
}
