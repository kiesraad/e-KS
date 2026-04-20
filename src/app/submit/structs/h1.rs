use crate::{
    AppError, AppStore, ElectionConfig,
    candidate_lists::{CandidateListId, FullCandidateList},
    core::{ElectionType, ModelLocale, Pdf},
    submit::structs::{
        TypstCandidate, TypstDatetime, TypstElectoralDistricts, TypstPerson, ordered_candidates,
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

        let list_submitter = store.get_list_submitter();
        if !list_submitter.is_complete() {
            return Err(AppError::IncompleteData("Missing list submitter"));
        }

        let substitute_submitter = store
            .get_substitute_submitters()
            .into_iter()
            .map(TypstPerson::try_from)
            .collect::<Result<Vec<_>, _>>()?;

        Ok(Self {
            election_name: election.formal_title(locale),
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
            list_submitter: list_submitter.try_into()?,
            substitute_submitter,
            timestamp: TypstDatetime::now(),
            locale,
            filename,
        })
    }
}
