use crate::{
    core::{ElectionType, ModelLocale, Pdf},
    submit::{
        DocumentData,
        structs::{
            TypstCandidate, TypstDatetime, TypstElectoralDistricts, TypstPerson,
            typst_name_authorisation::TypstNameAuthorisation,
        },
    },
};
use serde::Serialize;

/// Either an H3-1 or H3-2 model, depending on the number of name authorisations provided.
#[derive(Debug, Serialize)]
pub struct H3<'a> {
    election_name: String,
    election_type: ElectionType,
    electoral_districts: &'a TypstElectoralDistricts,
    designation: &'a str,
    candidates: &'a Vec<TypstCandidate>,
    list_submitter: &'a TypstPerson,
    name_authorisations: &'a Vec<TypstNameAuthorisation>,
    timestamp: &'a TypstDatetime,
    locale: ModelLocale,
    event_id: usize,
    sha_hash: &'a str,
    filename: &'static str,
    template_name: &'static str,
}

impl Pdf for H3<'_> {
    fn typst_template_name(&self) -> &'static str {
        self.template_name
    }

    fn filename(&self) -> &str {
        self.filename
    }
}

impl<'a> From<&'a DocumentData> for H3<'a> {
    fn from(data: &'a DocumentData) -> Self {
        let locale = data.locale;
        let election = data.election;

        let use_h3_2 = data.name_authorisations.len() > 1;

        let filename = match (locale, use_h3_2) {
            (ModelLocale::Nl, false) => "h3-1-aanduiding.pdf",
            (ModelLocale::Fry, false) => "h3-1-oantsjutting.pdf",
            (ModelLocale::Nl, true) => "h3-2-samengevoegde-aanduiding.pdf",
            (ModelLocale::Fry, true) => "h3-2-gearfoege-oantsjutting.pdf",
        };

        let template_name = if use_h3_2 {
            "model-h3-2.typ"
        } else {
            "model-h3-1.typ"
        };

        Self {
            election_name: election.formal_title(locale),
            election_type: election.election_type(),
            electoral_districts: &data.electoral_districts,
            designation: &data.designation,
            candidates: &data.ordered_candidates,
            list_submitter: &data.list_submitter,
            name_authorisations: &data.name_authorisations,
            timestamp: &data.timestamp,
            locale,
            event_id: data.event_id,
            sha_hash: &data.event_hash,
            filename,
            template_name,
        }
    }
}
