use crate::{
    AppError, AppStore, Context, ElectionConfig, TypstRenderer,
    candidate_lists::{CandidateListId, FullCandidateList},
    common::{PreviousElectionResults, Problematic},
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
    utils::{format_hash, no_cache_headers, slugify_teletex},
};
use axum::{body::Body, http::HeaderValue, response::IntoResponse};
use tokio::io::duplex;
use tokio_util::io::ReaderStream;
use tracing::error;

pub const ZIP_CONTENT_TYPE: &str = "application/zip";

pub struct DocumentData {
    pub list_id: CandidateListId,
    pub folder_name: Option<String>,
    pub locale: ModelLocale,
    pub timestamp: TypstDatetime,
    pub election: ElectionConfig,
    pub electoral_districts: TypstElectoralDistricts,
    pub detailed_candidates: Vec<TypstDetailedCandidate>,
    pub ordered_candidates: Vec<TypstCandidate>,
    pub designation: String,
    pub legal_name: String,
    pub previously_seated: bool,
    pub list_submitter: TypstPerson,
    pub substitute_submitters: Vec<TypstPerson>,
    pub authorised_agent: TypstAuthorisedAgent,
    pub event_id: usize,
    pub event_hash: String,
    nomination: Eml210,
}

impl DocumentData {
    pub fn archive_filename(&self) -> String {
        let name_slug = slugify_teletex(&self.designation, true);
        let mut election_slug = self.election.code().to_lowercase();
        if let Some(region) = self.election.region_code() {
            election_slug.push_str(&region.to_lowercase());
        }
        let version = self.event_id;

        if self.locale == ModelLocale::Fry {
            format!("{name_slug}-{election_slug}-v{version}-fry.zip")
        } else {
            format!("{name_slug}-{election_slug}-v{version}.zip")
        }
    }

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

        let event_id = store.current_event_id();
        let event_hash = store.current_event_hash();

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
        // Missing designation prevents export
        let designation = group
            .display_name
            .as_ref()
            .ok_or(AppError::IncompleteData(
                "Missing registered designation from political group",
            ))?
            .to_string();
        // Missing statutory name does not prevent export
        let legal_name = group
            .legal_name
            .as_ref()
            .map(|name| name.to_string())
            .unwrap_or_default();

        let list_submitter = store.get_list_submitter();
        if list_submitter.is_empty() || !list_submitter.is_all_good() {
            return Err(AppError::IncompleteData("Incomplete list submitter"));
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
        let folder_name = format!(
            "{}-{}",
            match locale {
                ModelLocale::Nl => "kieskring",
                ModelLocale::Fry => "kiesrunte",
            },
            list.districts_codes()
        );

        Ok(Self {
            list_id,
            folder_name: Some(folder_name),
            locale,
            timestamp: TypstDatetime::now(),
            election,
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
            event_id,
            event_hash: format_hash(&event_hash, true),
            nomination,
        })
    }

    pub async fn from_store_and_context(
        store: &AppStore,
        context: &Context,
        locale: ModelLocale,
    ) -> Result<Vec<Self>, AppError> {
        let list_ids = store
            .get_candidate_lists()
            .into_iter()
            .map(|list| list.id)
            .collect::<Vec<_>>();

        if list_ids.is_empty() {
            return Err(AppError::IncompleteData("No candidate lists"));
        }

        let bundles = if list_ids.len() == 1 {
            let mut bundle = Self::new(&store, &context, list_ids[0], locale)?;
            bundle.folder_name = None;

            vec![bundle]
        } else {
            list_ids
                .iter()
                .map(|&list_id| Self::new(&store, &context, list_id, locale))
                .collect::<Result<Vec<_>, _>>()?
        };
        Ok(bundles)
    }

    pub async fn to_zip_response(bundles: Vec<Self>, filename: String, renderer: TypstRenderer) -> Result<impl IntoResponse, AppError> {
        let headers = no_cache_headers::generate_attachment_headers(
            &filename,
            HeaderValue::from_static(ZIP_CONTENT_TYPE),
        )?;

        let (reader, writer) = duplex(64 * 1024);
        let body = Body::from_stream(ReaderStream::new(reader));

        tokio::spawn(async move {
            let mut zipper = ZipResponseWriter::new(writer);

            for bundle in bundles {
                let list_id = bundle.list_id;
                if let Err(err) = bundle.write_zip(&renderer, &mut zipper).await {
                    error!(
                        error = ?err,
                        list_id = %list_id,
                        "failed to stream submit documents zip"
                    );
                    return;
                }
            }

            if let Err(err) = zipper.finish().await {
                error!(error = ?err, "failed to finalize submit documents zip");
            }
        });

        Ok((headers, body).into_response())
    }

    async fn write_zip(
        self,
        renderer: &TypstRenderer,
        writer: &mut ZipResponseWriter<tokio::io::DuplexStream>,
    ) -> Result<(), AppError> {
        let h1 = H1::from(&self);
        let h1_path = self.zip_path(h1.filename());
        writer
            .add_file(&h1_path, &h1.generate_bytes(renderer).await?)
            .await?;

        let h3_1 = H31::from(&self);
        let h3_1_path = self.zip_path(h3_1.filename());
        writer
            .add_file(&h3_1_path, &h3_1.generate_bytes(renderer).await?)
            .await?;

        let h4 = H4::from(&self);
        let h4_path = self.zip_path(h4.filename());
        writer
            .add_file(&h4_path, &h4.generate_bytes(renderer).await?)
            .await?;

        for candidate in self.detailed_candidates.iter() {
            let h9 = H9::from((&self, candidate));
            let filename = self.zip_path(&format!(
                "h9-{}/{}",
                match self.locale {
                    ModelLocale::Nl => "instemmingsverklaringen",
                    ModelLocale::Fry => "ynstimmingsferklearrings",
                },
                h9.filename()
            ));
            writer
                .add_file(&filename, &h9.generate_bytes(renderer).await?)
                .await?;
        }

        writer
            .add_file(&self.zip_path("eml210.eml.xml"), &self.nomination.build()?)
            .await?;

        Ok(())
    }

    fn zip_path(&self, relative_path: &str) -> String {
        match &self.folder_name {
            Some(folder_name) => format!("{folder_name}/{relative_path}"),
            None => relative_path.to_string(),
        }
    }
}
