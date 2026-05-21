use crate::{
    core::{ElectionType, ModelLocale, Pdf},
    submit::{
        DocumentData,
        structs::{TypstCandidate, TypstDatetime, TypstElectoralDistricts, TypstPerson},
    },
};
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct H1<'a> {
    election_name: String,
    election_type: ElectionType,
    electoral_districts: &'a TypstElectoralDistricts,
    designation: &'a str,
    candidates: &'a Vec<TypstCandidate>,
    previously_seated: bool,
    list_submitter: &'a TypstPerson,
    substitute_submitters: &'a Vec<TypstPerson>,
    timestamp: &'a TypstDatetime,
    locale: ModelLocale,
    event_id: usize,
    sha_hash: &'a str,
    filename: &'static str,
}

impl Pdf for H1<'_> {
    fn typst_template_name(&self) -> &'static str {
        "model-h1.typ"
    }

    fn filename(&self) -> &str {
        self.filename
    }
}

impl<'a> From<&'a DocumentData> for H1<'a> {
    fn from(data: &'a DocumentData) -> Self {
        let locale = data.locale;
        let election = data.election;

        let filename = match locale {
            ModelLocale::Nl => "h1-kandidatenlijst.pdf",
            ModelLocale::Fry => "h1-kandidatelist.pdf",
        };

        Self {
            election_name: election.formal_title(locale),
            election_type: election.election_type(),
            electoral_districts: &data.electoral_districts,
            designation: &data.designation,
            candidates: &data.ordered_candidates,
            previously_seated: data.previously_seated,
            list_submitter: &data.list_submitter,
            substitute_submitters: &data.substitute_submitters,
            timestamp: &data.timestamp,
            locale,
            event_id: data.event_id,
            sha_hash: &data.event_hash,
            filename,
        }
    }
}
