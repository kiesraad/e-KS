use crate::{
    AppError, AppStore, Context, ElectionConfig, TypstRenderer,
    candidate_lists::{CandidateList, CandidateListId, FullCandidateList},
    common::PreviousElectionResults,
    core::{ModelLocale, Pdf, ZipResponseWriter},
    submit::structs::{
        eml210::Eml210,
        h1::H1,
        h3_1::H31,
        h4::H4,
        h9::H9,
        typst_authorised_agent::TypstAuthorisedAgent,
        typst_candidate::{TypstCandidate, ordered_candidates},
        typst_datetime::TypstDatetime,
        typst_detailed_candidate::TypstDetailedCandidate,
        typst_electoral_districts::TypstElectoralDistricts,
        typst_person::TypstPerson,
    },
};

pub struct DocumentData {
    pub filename: String,
    pub locale: ModelLocale,
    pub timestamp: TypstDatetime,
    pub election: ElectionConfig,
    pub list: CandidateList,
    pub electoral_districts: TypstElectoralDistricts,
    pub detailed_candidates: Vec<TypstDetailedCandidate>,
    pub ordered_candidates: Vec<TypstCandidate>,
    pub designation: String,
    pub legal_name: String,
    pub previously_seated: bool,
    pub list_submitter: TypstPerson,
    pub substitute_submitters: Vec<TypstPerson>,
    pub authorised_agent: TypstAuthorisedAgent,
    nomination: Eml210,
}

impl DocumentData {
    /// Collect all the necessary data to render the models and the exported EML.
    ///
    /// Collecting the data first prevents errors popping up while the ZIP is streaming,
    /// and it is more efficient because we only collect everything once.
    pub fn new(
        store: &AppStore,
        context: &Context,
        list_id: CandidateListId,
        locale: ModelLocale,
    ) -> Result<Self, AppError> {
        let election = context.election;
        if !election.frisian_export_allowed() && locale == ModelLocale::Fry {
            return Err(AppError::UserError(
                "Frisian export not allowed for this election".to_string(),
            ));
        }

        let FullCandidateList {
            list,
            mut candidates,
        } = FullCandidateList::get(store, list_id)?;
        let ordered_candidates = ordered_candidates(&mut candidates, locale)?;
        let detailed_candidates = candidates
            .iter()
            .map(|c| TypstDetailedCandidate::try_from(c, locale))
            .collect::<Result<Vec<_>, _>>()?;

        let electoral_districts = TypstElectoralDistricts::from(&list, &context.election, locale);

        let group = store.get_political_group();
        let designation = group
            .display_name
            .as_ref()
            .ok_or(AppError::IncompleteData(
                "Missing registered designation from political group",
            ))?
            .to_string();
        let legal_name = group
            .legal_name
            .as_ref()
            .ok_or(AppError::IncompleteData(
                "Missing statutory name from political group",
            ))?
            .to_string();

        let list_submitter = store.get_list_submitter();
        if !list_submitter.is_complete() {
            return Err(AppError::IncompleteData("Missing list submitter"));
        }
        let list_submitter = list_submitter.try_into()?;

        let substitute_submitters = store
            .get_substitute_submitters()
            .into_iter()
            .map(TypstPerson::try_from)
            .collect::<Result<Vec<_>, _>>()?;

        let authorised_agents = store.get_authorised_agents();
        if authorised_agents.len() != 1 {
            return Err(AppError::IncompleteData("Expected 1 authorised agent"));
        }
        let authorised_agent = (&authorised_agents[0]).into();

        let nomination = Eml210::new(store, &election, &group, list_id, locale)?;

        let filename = if list.contains_all_districts(&election) {
            format!("documents-{locale}.zip")
        } else {
            format!("documents-{}-{locale}.zip", list.districts_codes())
        };

        Ok(Self {
            filename,
            locale,
            timestamp: TypstDatetime::now(),
            election,
            list,
            electoral_districts,
            detailed_candidates,
            ordered_candidates,
            designation,
            legal_name,
            previously_seated: group
                .previous_election_results
                .is_some_and(|r| r != PreviousElectionResults::ZeroSeats),
            list_submitter,
            substitute_submitters,
            authorised_agent,
            nomination,
        })
    }

    pub async fn write_zip(
        self,
        renderer: TypstRenderer,
        mut writer: ZipResponseWriter<tokio::io::DuplexStream>,
    ) -> Result<(), AppError> {
        let h1 = H1::from(&self);
        writer
            .add_file(h1.filename(), &h1.generate_bytes(&renderer).await?)
            .await?;

        let h3_1 = H31::from(&self);
        writer
            .add_file(h3_1.filename(), &h3_1.generate_bytes(&renderer).await?)
            .await?;

        let h4 = H4::from(&self);
        writer
            .add_file(h4.filename(), &h4.generate_bytes(&renderer).await?)
            .await?;

        for candidate in self.detailed_candidates.iter() {
            let h9 = H9::from((&self, candidate));
            let filename = format!("model-h9/{}", h9.filename());
            writer
                .add_file(&filename, &h9.generate_bytes(&renderer).await?)
                .await?;
        }

        writer
            .add_file("eml210.eml.xml", &self.nomination.build()?)
            .await?;

        writer.finish().await?;

        Ok(())
    }
}
