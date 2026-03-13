use crate::{
    AppError, AppStore, ElectionConfig,
    candidate_lists::FullCandidateList,
    core::{ElectionType, ModelLocale, Pdf},
    submit::structs::{
        ElectoralDistricts, TypstCandidate, TypstDatetime, TypstPerson, ordered_candidates,
        substitute_submitter_from_ids,
    },
};
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct H1 {
    election_name: String,
    election_type: ElectionType,
    electoral_districts: ElectoralDistricts,
    designation: String,
    candidates: Vec<TypstCandidate>,
    previously_seated: bool,
    list_submitter: TypstPerson,
    substitute_submitter: Vec<TypstPerson>,
    timestamp: TypstDatetime,
    locale: ModelLocale,
}

impl Pdf for H1 {
    fn typst_template_name(&self) -> String {
        format!("model-h1-{}.typ", self.locale)
    }

    fn filename(&self) -> String {
        format!(
            "model-h1-{}-({}).pdf",
            self.locale, self.electoral_districts
        )
    }
}

impl H1 {
    pub fn new(
        store: &AppStore,
        FullCandidateList {
            list,
            mut candidates,
        }: FullCandidateList,
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
            previously_seated: true,
            list_submitter: store
                .get_list_submitter(
                    list.list_submitter_id
                        .ok_or(AppError::IncompleteData("Missing list submitter"))?,
                )?
                .try_into()?,
            substitute_submitter: substitute_submitter_from_ids(&list, store.clone())?,
            timestamp: TypstDatetime::now(),
            locale,
        })
    }
}
