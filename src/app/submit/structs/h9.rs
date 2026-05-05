use serde::Serialize;

use crate::{
    core::{ElectionType, ModelLocale, Pdf},
    submit::{
        DocumentData,
        structs::{
            TypstCandidate, TypstDatetime, TypstElectoralDistricts,
            typst_detailed_candidate::TypstDetailedCandidate,
        },
    },
    utils::slugify_teletex,
};

#[derive(Debug, Serialize)]
pub struct H9<'a> {
    election_name: String,
    election_type: ElectionType,
    electoral_districts: &'a TypstElectoralDistricts,
    designation: &'a str,
    candidates: &'a Vec<TypstCandidate>,
    detailed_candidate: &'a TypstDetailedCandidate,
    timestamp: &'a TypstDatetime,
    locale: ModelLocale,
    filename: String,
}

impl Pdf for H9<'_> {
    fn typst_template_name(&self) -> &'static str {
        "model-h9.typ"
    }

    fn filename(&self) -> &str {
        &self.filename
    }
}

impl<'a> From<(&'a DocumentData, &'a TypstDetailedCandidate)> for H9<'a> {
    fn from((data, candidate): (&'a DocumentData, &'a TypstDetailedCandidate)) -> Self {
        let locale = data.locale;
        let election = data.election;

        let filename = format!(
            "model-h9-{}-{}.pdf",
            slugify_teletex(&candidate.candidate.last_name, true),
            candidate.candidate.position
        );

        Self {
            election_name: election.formal_title(locale),
            election_type: election.election_type(),
            electoral_districts: &data.electoral_districts,
            designation: &data.designation,
            candidates: &data.ordered_candidates,
            detailed_candidate: candidate,
            timestamp: &data.timestamp,
            locale,
            filename,
        }
    }
}
