use crate::{
    AppError, AppStore, ElectionConfig,
    candidate_lists::FullCandidateList,
    core::{ElectionType, ModelLocale, Pdf},
    submit::structs::{
        ElectoralDistricts, TypstCandidate, TypstDatetime, TypstPerson, ordered_candidates,
        typst_authorised_agent::TypstAuthorisedAgent,
    },
};
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct H31 {
    election_name: String,
    election_type: ElectionType,
    electoral_districts: ElectoralDistricts,
    designation: String,
    legal_name: String,
    candidates: Vec<TypstCandidate>,
    list_submitter: TypstPerson,
    authorised_agent: TypstAuthorisedAgent,
    timestamp: TypstDatetime,
    locale: ModelLocale,
}

impl Pdf for H31 {
    fn typst_template_name(&self) -> String {
        "model-h3-1.typ".to_owned()
    }

    fn filename(&self) -> String {
        format!(
            "model-h3-1-{}-({}).pdf",
            self.locale, self.electoral_districts
        )
    }
}

impl H31 {
    pub fn new(
        store: &AppStore,
        FullCandidateList {
            list,
            mut candidates,
        }: FullCandidateList,
        election: &ElectionConfig,
        locale: ModelLocale,
    ) -> Result<Self, AppError> {
        let political_group = store.get_political_group();
        let authorised_agents = store.get_authorised_agents();
        if authorised_agents.len() != 1 {
            return Err(AppError::IncompleteData("Expected 1 authorised agent"));
        }

        Ok(Self {
            election_name: election.title(locale.into()).to_string(),
            election_type: election.election_type(),
            electoral_districts: ElectoralDistricts::from(&list, election, locale),
            designation: political_group
                .display_name
                .ok_or(AppError::IncompleteData(
                    "Missing registered designation from political group",
                ))?
                .to_string(),
            legal_name: political_group
                .legal_name
                .ok_or(AppError::IncompleteData(
                    "Missing statutory name from political group",
                ))?
                .to_string(),
            candidates: ordered_candidates(&mut candidates, locale)?,
            list_submitter: store
                .get_list_submitter(
                    list.list_submitter_id
                        .ok_or(AppError::IncompleteData("Missing list submitter"))?,
                )?
                .try_into()?,
            authorised_agent: (&authorised_agents[0]).into(),
            timestamp: TypstDatetime::now(),
            locale,
        })
    }
}
