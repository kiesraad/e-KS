use crate::{
    AppError, AppStore, Context, ElectionConfig, TypstRenderer,
    candidate_lists::{CandidateListId, FullCandidateList},
    common::{HasSeverity, Problematic, Severity},
    core::{ModelLocale, Pdf, ZipResponseWriter},
    finalise::structs::eml210::Eml210,
    list_designation::ListDesignation,
    typst::{
        H1, H3, H4, H9, TypstCandidate, TypstDatetime, TypstDetailedCandidate,
        TypstElectoralDistricts, TypstNameAuthorisation, TypstPerson, TypstPgModelData,
    },
    utils::{format_hash, no_cache_headers, slugify_teletex},
};
use axum::{
    body::Body,
    http::HeaderValue,
    response::{IntoResponse, Response},
};
use tokio::io::duplex;
use tokio_util::io::ReaderStream;
use tracing::error;

pub const ZIP_CONTENT_TYPE: &str = "application/zip";

pub struct DocumentData {
    pub list_id: CandidateListId,
    pub folder_name: Option<String>,
    pub election: ElectionConfig,
    pub model_data: TypstPgModelData,
    pub detailed_candidates: Vec<TypstDetailedCandidate>,
    pub previously_seated: bool,
    pub list_designation: ListDesignation,
    pub list_submitter: TypstPerson,
    pub substitute_submitters: Vec<TypstPerson>,
    pub name_authorisations: Vec<TypstNameAuthorisation>,
    nomination: Eml210,
}

impl DocumentData {
    pub fn archive_filename(&self) -> String {
        let mut election_slug = self.election.code().to_lowercase();
        if let Some(region) = self.election.region_code() {
            election_slug.push_str(&region.to_lowercase());
        }
        let version = self.model_data.event_id;

        let name_slug = if self.list_designation == ListDesignation::Blank {
            "blanco".to_string()
        } else {
            slugify_teletex(&self.model_data.designation, true)
        };

        if self.model_data.locale == ModelLocale::Fry {
            format!("{name_slug}-{election_slug}-v{version}-fry.zip")
        } else {
            format!("{name_slug}-{election_slug}-v{version}.zip")
        }
    }

    /// Get a list of `TypstNameAuthorisation` with the right number of authorisations based on
    /// the type of list designation:
    ///
    /// - Blank lists always have 0 name authorisations -> No H3-1 or H3-2
    /// - Combined lists have at least 2 name authorisations -> H3-2
    /// - Standalone lists always have 1 name authorisation -> H3-1
    ///
    /// If there are fewer name authorisations than required, we add fill-ins that show up as
    /// empty spaces on the models.
    fn name_authorisations_with_fill_ins(
        store: &AppStore,
    ) -> Result<Vec<TypstNameAuthorisation>, AppError> {
        let name_authorisations = store.get_name_authorisations();

        match store.get_political_group().list_designation {
            Some(ListDesignation::Blank) => Ok(Vec::new()),
            Some(ListDesignation::Combined) => {
                let mut auths: Vec<TypstNameAuthorisation> =
                    name_authorisations.iter().map(Into::into).collect();

                while auths.len() < 2 {
                    auths.push(TypstNameAuthorisation::default());
                }

                Ok(auths)
            }
            _ => {
                if name_authorisations.len() > 1 {
                    return Err(AppError::IncompleteData(
                        "Expected no more than 1 name authorisation",
                    ));
                }

                let auth = name_authorisations
                    .first()
                    .map(Into::into)
                    .unwrap_or_default();

                Ok(vec![auth])
            }
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

        let FullCandidateList { list, candidates } = FullCandidateList::get(store, list_id)?;
        let mut candidates = candidates.into_iter().map(|c| c.data).collect::<Vec<_>>();

        let ordered_candidates = TypstCandidate::ordered(&mut candidates, locale)?;
        let detailed_candidates = candidates
            .iter()
            .map(|c| TypstDetailedCandidate::try_from(c, locale))
            .collect::<Result<Vec<_>, _>>()?;

        let electoral_districts = TypstElectoralDistricts::from(&list, &context.election, locale);

        let group = store.get_political_group();
        let designation = group.pg_display_name()?;

        let list_submitter = store.get_list_submitter();
        if list_submitter.is_empty()
            || list_submitter
                .get_problems(())
                .has_severity_or_higher(Severity::Error)
        {
            return Err(AppError::IncompleteData("Incomplete list submitter"));
        }
        let list_submitter = list_submitter.try_into()?;

        let substitute_submitters = store
            .get_substitute_submitters()
            .into_iter()
            .map(TypstPerson::try_from)
            .collect::<Result<Vec<_>, _>>()?;

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
            election,
            model_data: TypstPgModelData {
                election_name: election.formal_title(locale),
                election_type: election.election_type(),
                electoral_districts,
                designation,
                candidates: ordered_candidates,
                timestamp: TypstDatetime::now(),
                locale,
                event_id,
                sha_hash: format_hash(&event_hash, true),
            },
            detailed_candidates,
            previously_seated: group.was_previously_seated(),
            list_designation: group.list_designation.unwrap_or_default(),
            list_submitter,
            substitute_submitters,
            name_authorisations: Self::name_authorisations_with_fill_ins(store)?,
            nomination,
        })
    }

    pub fn from_store_and_context(
        store: &AppStore,
        context: &Context,
        locale: ModelLocale,
    ) -> Result<(Vec<Self>, String), AppError> {
        let list_ids = store
            .get_candidate_lists()
            .into_iter()
            .map(|list| list.id)
            .collect::<Vec<_>>();

        if list_ids.is_empty() {
            return Err(AppError::IncompleteData("No candidate lists"));
        }

        let bundles = if list_ids.len() == 1 {
            let mut bundle = Self::new(store, context, list_ids[0], locale)?;
            bundle.folder_name = None;

            vec![bundle]
        } else {
            list_ids
                .iter()
                .map(|&list_id| Self::new(store, context, list_id, locale))
                .collect::<Result<Vec<_>, _>>()?
        };
        let Some(document_data) = bundles.first() else {
            return Err(AppError::IncompleteData("No candidate lists"));
        };

        let filename = document_data.archive_filename();
        Ok((bundles, filename))
    }

    /// Record a document download as a `DownloadFile` audit event and stream
    /// the bundles as a zip response.
    ///
    /// The audit event is written to `event_store`. `document_store` is the
    /// (possibly historical) store the bundles were generated from; it is only
    /// used for the candidate-list count in the log line, which differs from
    /// `event_store` when serving documents for a past event.
    pub async fn serve_download(
        bundles: Vec<Self>,
        filename: String,
        download_path: String,
        event_store: &AppStore,
        document_store: &AppStore,
        renderer: TypstRenderer,
    ) -> Result<Response, AppError> {
        tracing::info!(
            filename,
            content_type = ZIP_CONTENT_TYPE,
            lists = document_store.get_candidate_list_count(),
            "file download served",
        );

        event_store
            .update(crate::AppEvent::DownloadFile {
                file_name: filename.clone(),
                download_path,
            })
            .await?;

        Self::to_zip_response(bundles, filename, renderer).map(IntoResponse::into_response)
    }

    pub fn to_zip_response(
        bundles: Vec<Self>,
        filename: String,
        renderer: TypstRenderer,
    ) -> Result<impl IntoResponse, AppError> {
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
                error!(error = ?err, "failed to finalise submit documents zip");
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

        if self.list_designation != ListDesignation::Blank {
            let h3 = H3::from(&self);
            let h3_path = self.zip_path(h3.filename());
            writer
                .add_file(&h3_path, &h3.generate_bytes(renderer).await?)
                .await?;
        }

        if !self.previously_seated {
            let h4 = H4::from(&self);
            let h4_path = self.zip_path(h4.filename());
            writer
                .add_file(&h4_path, &h4.generate_bytes(renderer).await?)
                .await?;
        }

        for candidate in self.detailed_candidates.iter() {
            let h9 = H9::from((&self, candidate));
            let filename = self.zip_path(format!(
                "h9-{}/{}",
                match self.model_data.locale {
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
            .add_file(
                &self.zip_path("eml210.eml.xml".to_string()),
                &self.nomination.build()?,
            )
            .await?;

        Ok(())
    }

    fn zip_path(&self, relative_path: String) -> String {
        match &self.folder_name {
            Some(folder_name) => format!("{folder_name}/{relative_path}"),
            None => relative_path,
        }
    }
}
