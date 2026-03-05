use serde::Serialize;

use crate::{
    AppError, AppStore, ElectionConfig,
    candidate_lists::FullCandidateList,
    candidates::Candidate,
    core::{ElectionType, ModelLocale, Pdf},
    submit::structs::{
        ElectoralDistricts, TypstCandidate, TypstDatetime, typst_candidate::ordered_candidates,
        typst_detailed_candidate::TypstDetailedCandidate,
    },
};

#[derive(Debug, Serialize)]
pub struct H9 {
    election_name: String,
    election_type: ElectionType,
    electoral_districts: ElectoralDistricts,
    designation: String,
    candidates: Vec<TypstCandidate>,
    detailed_candidate: TypstDetailedCandidate,
    timestamp: TypstDatetime,
    locale: ModelLocale,
}

impl Pdf for H9 {
    fn typst_template_name(&self) -> String {
        format!("model-h9-{}.typ", self.locale)
    }

    fn filename(&self) -> String {
        format!(
            "model-h9-{}-{}-({}).pdf",
            self.locale,
            self.detailed_candidate
                .candidate
                .last_name
                .replace(" ", "-"),
            self.detailed_candidate.candidate.position
        )
    }
}

impl H9 {
    pub fn new(
        store: &AppStore,
        FullCandidateList {
            list,
            mut candidates,
        }: FullCandidateList,
        candidate: Candidate,
        election: &ElectionConfig,
        locale: ModelLocale,
    ) -> Result<Self, AppError> {
        Ok(Self {
            election_name: election.title(locale.into()).to_string(),
            election_type: election.election_type(),
            electoral_districts: ElectoralDistricts::from(&list, election, locale),
            designation: store
                .get_political_group()
                .display_name
                .ok_or(AppError::IncompleteData(
                    "Missing registered designation from political group",
                ))?
                .to_string(),
            candidates: ordered_candidates(&mut candidates, locale)?,
            detailed_candidate: TypstDetailedCandidate::try_from(&candidate, locale)?,
            timestamp: TypstDatetime::now(),
            locale,
        })
    }
}
