use serde::Serialize;

use crate::{
    AppError, AppStore, ElectionConfig,
    candidate_lists::{CandidateListId, FullCandidateList},
    core::{ElectionType, ModelLocale, Pdf},
    submit::structs::{
        typst_candidate::{TypstCandidate, ordered_candidates},
        typst_datetime::TypstDatetime,
        typst_electoral_districts::TypstElectoralDistricts,
    },
};

#[derive(Debug, Serialize)]
pub struct H4 {
    election_name: String,
    election_type: ElectionType,
    electoral_districts: TypstElectoralDistricts,
    designation: String,
    candidates: Vec<TypstCandidate>,
    timestamp: TypstDatetime,
    locale: ModelLocale,
    filename: String,
}

impl Pdf for H4 {
    fn typst_template_name(&self) -> &'static str {
        "model-h4.typ"
    }

    fn filename(&self) -> &str {
        &self.filename
    }
}

impl H4 {
    pub fn new(
        store: &AppStore,
        list_id: CandidateListId,
        election: &ElectionConfig,
        locale: ModelLocale,
    ) -> Result<Self, AppError> {
        let FullCandidateList {
            list,
            mut candidates,
        } = FullCandidateList::get(store, list_id)?;

        let filename = if list.electoral_districts.len() == 1 {
            format!(
                "model-h4-({}).pdf",
                list.electoral_districts[0].title(locale.into())
            )
        } else {
            "model-h4.pdf".to_string()
        };

        Ok(Self {
            election_name: election.title(locale.into()).to_string(),
            election_type: election.election_type(),
            electoral_districts: TypstElectoralDistricts::from(&list, election, locale),
            designation: store
                .get_political_group()
                .display_name
                .ok_or(AppError::IncompleteData(
                    "Missing registered designation from political group",
                ))?
                .to_string(),
            candidates: ordered_candidates(&mut candidates, locale)?,
            timestamp: TypstDatetime::now(),
            locale,
            filename,
        })
    }
}
