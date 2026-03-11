use serde::Serialize;

use crate::{
    AppError, AppStore, ElectionConfig,
    candidate_lists::CandidateList,
    candidates::Candidate,
    core::{ElectionType, ModelLocale, Pdf},
    submit::structs::{
        ElectoralDistricts, TypstCandidate, TypstDatetime,
        typst_detailed_candidate::TypstDetailedCandidate,
    },
};

#[derive(Debug, Serialize)]
pub struct H9<'zip> {
    election_name: String,
    election_type: ElectionType,
    pub electoral_districts: ElectoralDistricts,
    designation: String,
    candidates: &'zip Vec<TypstCandidate>,
    detailed_candidate: TypstDetailedCandidate,
    timestamp: TypstDatetime,
    locale: ModelLocale,
}

impl<'zip> Pdf for H9<'zip> {
    fn typst_template_name(&self) -> String {
        format!("model-h9-{}.typ", self.locale)
    }

    fn filename(&self) -> String {
        format!(
            "model-h9-{}-{}-(#{}).pdf",
            self.locale,
            self.detailed_candidate
                .candidate
                .last_name
                .replace(" ", "-"),
            self.detailed_candidate.candidate.position
        )
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
        Ok(Self {
            election_name: election.title(locale.into()).to_string(),
            election_type: election.election_type(),
            electoral_districts: ElectoralDistricts::from(candidate_list, election, locale),
            designation: store
                .get_political_group()
                .display_name
                .ok_or(AppError::IncompleteData(
                    "Missing registered designation from political group",
                ))?
                .to_string(),
            candidates: ordered_candidates,
            detailed_candidate: TypstDetailedCandidate::try_from(&candidate, locale)?,
            timestamp: TypstDatetime::now(),
            locale,
        })
    }
}
