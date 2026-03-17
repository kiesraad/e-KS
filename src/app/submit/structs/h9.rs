use serde::Serialize;

use crate::{
    AppError, AppStore, ElectionConfig,
    candidate_lists::CandidateList,
    candidates::Candidate,
    core::{ElectionType, ModelLocale, Pdf},
    submit::structs::{
        TypstCandidate, TypstDatetime, TypstElectoralDistricts,
        typst_detailed_candidate::TypstDetailedCandidate,
    },
    utils::slugify_teletex,
};

#[derive(Debug, Serialize)]
pub struct H9<'zip> {
    election_name: String,
    election_type: ElectionType,
    pub electoral_districts: TypstElectoralDistricts,
    designation: String,
    candidates: &'zip Vec<TypstCandidate>,
    detailed_candidate: TypstDetailedCandidate,
    timestamp: TypstDatetime,
    locale: ModelLocale,
    filename: String,
}

impl<'zip> Pdf for H9<'zip> {
    fn typst_template_name(&self) -> &'static str {
        "model-h9.typ"
    }

    fn filename(&self) -> &str {
        &self.filename
    }
}

impl<'zip> H9<'zip> {
    pub fn new(
        store: &AppStore,
        candidate_list: &CandidateList,
        ordered_candidates: &'zip Vec<TypstCandidate>,
        candidate: Candidate,
        election: &ElectionConfig,
        locale: ModelLocale,
    ) -> Result<Self, AppError> {
        let detailed_candidate = TypstDetailedCandidate::try_from(&candidate, locale)?;
        let filename = format!(
            "model-h9-{}-{}.pdf",
            slugify_teletex(&detailed_candidate.candidate.last_name),
            detailed_candidate.candidate.position
        );

        Ok(Self {
            election_name: election.title(locale.into()).to_string(),
            election_type: election.election_type(),
            electoral_districts: TypstElectoralDistricts::from(candidate_list, election, locale),
            designation: store
                .get_political_group()
                .display_name
                .ok_or(AppError::IncompleteData(
                    "Missing registered designation from political group",
                ))?
                .to_string(),
            candidates: ordered_candidates,
            detailed_candidate,
            timestamp: TypstDatetime::now(),
            locale,
            filename,
        })
    }
}
