use crate::{
    core::{ElectionType, ModelLocale, Pdf},
    submit::{
        DocumentData,
        structs::{
            TypstCandidate, TypstDatetime, TypstElectoralDistricts, TypstPerson,
            typst_authorised_agent::TypstAuthorisedAgent,
        },
    },
};
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct H31<'a> {
    election_name: String,
    election_type: ElectionType,
    electoral_districts: &'a TypstElectoralDistricts,
    designation: &'a str,
    legal_name: &'a str,
    candidates: &'a Vec<TypstCandidate>,
    list_submitter: &'a TypstPerson,
    authorised_agent: &'a TypstAuthorisedAgent,
    timestamp: &'a TypstDatetime,
    locale: ModelLocale,
    event_id: usize,
    sha_hash: &'a str,
    filename: &'static str,
}

impl Pdf for H31<'_> {
    fn typst_template_name(&self) -> &'static str {
        "model-h3-1.typ"
    }

    fn filename(&self) -> &str {
        self.filename
    }
}

impl<'a> From<&'a DocumentData> for H31<'a> {
    fn from(data: &'a DocumentData) -> Self {
        let locale = data.locale;
        let election = data.election;

        let filename = match locale {
            ModelLocale::Nl => "h3-1-aanduiding.pdf",
            ModelLocale::Fry => "h3-1-oantsjutting.pdf",
        };

        Self {
            election_name: election.formal_title(locale),
            election_type: election.election_type(),
            electoral_districts: &data.electoral_districts,
            designation: &data.designation,
            legal_name: &data.legal_name,
            candidates: &data.ordered_candidates,
            list_submitter: &data.list_submitter,
            authorised_agent: &data.authorised_agent,
            timestamp: &data.timestamp,
            locale,
            event_id: data.event_id,
            sha_hash: &data.event_hash,
            filename,
        }
    }
}
