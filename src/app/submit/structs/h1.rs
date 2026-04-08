use crate::{
    AppError, AppStore, ElectionConfig,
    candidate_lists::{CandidateListId, FullCandidateList},
    core::{ElectionType, ModelLocale, Pdf},
    submit::structs::{
        TypstCandidate, TypstDatetime, TypstElectoralDistricts, TypstPerson, ordered_candidates,
        substitute_submitter_from_ids, typst_util,
    },
};
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct H1 {
    election_name: String,
    election_type: ElectionType,
    electoral_districts: TypstElectoralDistricts,
    designation: String,
    candidates: Vec<TypstCandidate>,
    previously_seated: bool,
    list_submitter: TypstPerson,
    substitute_submitter: Vec<TypstPerson>,
    timestamp: TypstDatetime,
    locale: ModelLocale,
    filename: String,
}

impl Pdf for H1 {
    fn typst_template_name(&self) -> &'static str {
        "model-h1.typ"
    }

    fn filename(&self) -> &str {
        &self.filename
    }
}

impl H1 {
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

        let filename = if list.contains_all_districts(election) {
            "model-h1.pdf".to_string()
        } else {
            format!("model-h1-{}.pdf", list.districts_codes())
        };

        let election_type = election.election_type();

        Ok(Self {
            election_name: typst_util::generate_election_title(&election_type, locale).to_string(),
            election_type,
            electoral_districts: TypstElectoralDistricts::from(&list, election, locale),
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
            filename,
        })
    }
}
