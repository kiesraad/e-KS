use crate::{
    AppError, AppStore, ElectionConfig,
    candidate_lists::{CandidateListId, FullCandidateList},
    core::{ElectionType, ModelLocale, Pdf},
    submit::structs::{
        TypstCandidate, TypstDatetime, TypstElectoralDistricts, TypstPerson, ordered_candidates,
        typst_authorised_agent::TypstAuthorisedAgent,
    },
};
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct H31 {
    election_name: String,
    election_type: ElectionType,
    electoral_districts: TypstElectoralDistricts,
    designation: String,
    legal_name: String,
    candidates: Vec<TypstCandidate>,
    list_submitter: TypstPerson,
    authorised_agent: TypstAuthorisedAgent,
    timestamp: TypstDatetime,
    locale: ModelLocale,
    filename: String,
}

impl Pdf for H31 {
    fn typst_template_name(&self) -> &'static str {
        "model-h3-1.typ"
    }

    fn filename(&self) -> &str {
        &self.filename
    }
}

impl H31 {
    pub fn new(
        store: &AppStore,
        list_id: CandidateListId,
        election: &ElectionConfig,
        locale: ModelLocale,
    ) -> Result<Self, AppError> {
        let political_group = store.get_political_group();
        let authorised_agents = store.get_authorised_agents();
        if authorised_agents.len() != 1 {
            return Err(AppError::IncompleteData("Expected 1 authorised agent"));
        }

        let FullCandidateList {
            list,
            mut candidates,
        } = FullCandidateList::get(store, list_id)?;

        let filename = if list.contains_all_districts(election) {
            "model-h3-1.pdf".to_string()
        } else {
            format!("model-h3-1-{}.pdf", list.districts_codes())
        };

        Ok(Self {
            election_name: election.title(locale.into()).to_string(),
            election_type: election.election_type(),
            electoral_districts: TypstElectoralDistricts::from(&list, election, locale),
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
            filename,
        })
    }
}
