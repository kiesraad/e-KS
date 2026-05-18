use axum::{extract::State, response::IntoResponse};

use crate::{
    AppError, AppEvent, AppStore, Context, TypstRenderer,
    submit::{DocumentData, pages::DownloadDocumentsPath, structs::documents::ZIP_CONTENT_TYPE},
};

pub async fn gen_documents(
    path @ DownloadDocumentsPath { locale }: DownloadDocumentsPath,
    store: AppStore,
    State(renderer): State<TypstRenderer>,
    context: Context,
) -> Result<impl IntoResponse, AppError> {
    let bundles = DocumentData::from_store_and_context(&store, &context, locale).await?;

    let Some(document_data) = bundles.first() else {
        return Err(AppError::IncompleteData("No candidate lists"));
    };

    let filename = document_data.archive_filename();

    tracing::info!(
        filename,
        content_type = ZIP_CONTENT_TYPE,
        lists = store.get_candidate_list_count(),
        "file download served",
    );

    store
        .update(AppEvent::DownloadFile {
            file_name: filename.clone(),
            download_path: path.to_string(),
        })
        .await?;

    DocumentData::to_zip_response(bundles, filename, renderer).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        ElectionConfig,
        authorised_agents::AuthorisedAgentId,
        core::ModelLocale,
        test_utils::{sample_authorised_agent, setup_documents_test_state},
    };
    #[cfg(feature = "embed-typst")]
    use crate::{
        common::{BsnOrNoneConfirmed, CountryCode, FullName},
        persons::Representative,
    };
    use axum::extract::State;

    #[tokio::test]
    async fn gen_documents_missing_list_submitter_returns_error() -> Result<(), AppError> {
        let (store, _, context) =
            setup_documents_test_state(1, 1, false, true, ElectionConfig::EK27).await?;
        let renderer = TypstRenderer::http("http://unused.test".to_string());

        let result = gen_documents(
            DownloadDocumentsPath {
                locale: crate::core::ModelLocale::Nl,
            },
            store,
            State(renderer),
            context,
        )
        .await;

        match result {
            Err(AppError::IncompleteData(_)) => {}
            _ => panic!("expected incomplete list submitter data error"),
        }

        Ok(())
    }

    #[tokio::test]
    async fn gen_documents_multiple_authorised_agents_return_error() -> Result<(), AppError> {
        let (store, _, context) =
            setup_documents_test_state(1, 1, true, true, ElectionConfig::EK27).await?;
        sample_authorised_agent(AuthorisedAgentId::new())
            .create(&store)
            .await?;
        let renderer = TypstRenderer::http("http://unused.test".to_string());

        let result = gen_documents(
            DownloadDocumentsPath {
                locale: crate::core::ModelLocale::Nl,
            },
            store,
            State(renderer),
            context,
        )
        .await;

        match result {
            Err(AppError::IncompleteData(message)) => {
                assert_eq!(message, "Expected 1 authorised agent")
            }
            _ => panic!("expected \"Expected 1 authorised agent\""),
        }

        Ok(())
    }

    #[tokio::test]
    async fn gen_documents_missing_designation_returns_error() -> Result<(), AppError> {
        let (store, _, context) =
            setup_documents_test_state(1, 1, true, true, ElectionConfig::EK27).await?;

        let mut political_group = store.get_political_group();
        political_group.display_name = None;
        political_group.legal_name = None;
        political_group.update(&store).await?;

        let renderer = TypstRenderer::http("http://unused.test".to_string());

        let result = gen_documents(
            DownloadDocumentsPath {
                locale: crate::core::ModelLocale::Nl,
            },
            store,
            State(renderer),
            context,
        )
        .await;

        match result {
            Err(AppError::IncompleteData(_)) => {}
            _ => panic!("expected missing data error"),
        }

        Ok(())
    }

    #[tokio::test]
    async fn gen_documents_disallowed_frisian_export_returns_error() -> Result<(), AppError> {
        let (store, _, context) =
            setup_documents_test_state(1, 1, true, true, ElectionConfig::PS27(crate::Province::GR))
                .await?;

        let renderer = TypstRenderer::http("http://unused.test".to_string());

        let result = gen_documents(
            DownloadDocumentsPath {
                locale: ModelLocale::Fry,
            },
            store,
            State(renderer),
            context,
        )
        .await;

        match result {
            Err(AppError::UserError(message)) => {
                assert_eq!(message, "Frisian export not allowed for this election")
            }
            _ => panic!("expected disallowed Frisian export error"),
        }

        Ok(())
    }

    #[cfg(feature = "embed-typst")]
    #[tokio::test]
    async fn gen_documents_returns_zip_response() -> Result<(), AppError> {
        use axum::{
            http::{StatusCode, header},
            response::IntoResponse,
        };
        use regex::Regex;

        let (store, list_ids, context) =
            setup_documents_test_state(2, 2, true, true, ElectionConfig::EK27).await?;
        let expected_folders = list_ids
            .iter()
            .map(|&list_id| {
                DocumentData::new(&store, &context, list_id, ModelLocale::Nl)
                    .map(|bundle| bundle.folder_name.expect("folder name"))
            })
            .collect::<Result<Vec<_>, _>>()?;
        let response = gen_documents(
            DownloadDocumentsPath {
                locale: crate::core::ModelLocale::Nl,
            },
            store,
            State(TypstRenderer::embedded(
                crate::utils::embed_typst::pdf_context(),
            )),
            context,
        )
        .await?
        .into_response();

        assert_eq!(response.status(), StatusCode::OK);
        let headers = response.headers().clone();
        assert_eq!(
            headers
                .get(header::CONTENT_TYPE)
                .expect("content type header"),
            "application/zip"
        );
        assert!(
            Regex::new("attachment; filename=\"kiesraad-demo-ek27-v\\d+\\.zip\"")
                .unwrap()
                .is_match(
                    headers
                        .get(header::CONTENT_DISPOSITION)
                        .expect("content disposition header")
                        .to_str()
                        .unwrap()
                )
        );
        assert_eq!(
            headers
                .get(header::CACHE_CONTROL)
                .expect("cache control header"),
            "no-store, no-cache, must-revalidate, max-age=0"
        );
        assert_eq!(
            headers.get(header::PRAGMA).expect("pragma header"),
            "no-cache"
        );
        assert_eq!(headers.get(header::EXPIRES).expect("expires header"), "0");

        let entry_names = crate::test_utils::zip_entry_names(response).await;
        for folder in expected_folders {
            assert!(entry_names.contains(&format!("{folder}/eml210.eml.xml")));
            assert!(
                entry_names
                    .iter()
                    .any(|name| name == &format!("{folder}/h1-kandidatenlijst.pdf"))
            );
            assert!(
                entry_names
                    .iter()
                    .any(|name| name == &format!("{folder}/h3-1-aanduiding.pdf"))
            );
            assert!(
                entry_names
                    .iter()
                    .any(|name| name == &format!("{folder}/h4-ondersteuningsverklaring.pdf"))
            );
            assert_eq!(
                entry_names
                    .iter()
                    .filter(|name| {
                        name.starts_with(&format!("{folder}/h9-instemmingsverklaringen/"))
                    })
                    .count(),
                2
            );
        }

        Ok(())
    }

    #[cfg(feature = "embed-typst")]
    #[tokio::test]
    async fn gen_documents_single_list_writes_files_at_zip_root() -> Result<(), AppError> {
        use axum::response::IntoResponse;

        let (store, _, context) =
            setup_documents_test_state(1, 2, true, true, ElectionConfig::EK27).await?;
        let response = gen_documents(
            DownloadDocumentsPath {
                locale: crate::core::ModelLocale::Nl,
            },
            store,
            State(TypstRenderer::embedded(
                crate::utils::embed_typst::pdf_context(),
            )),
            context,
        )
        .await?
        .into_response();

        let entry_names = crate::test_utils::zip_entry_names(response).await;
        assert!(entry_names.contains(&"eml210.eml.xml".to_string()));
        assert!(entry_names.contains(&"h1-kandidatenlijst.pdf".to_string()));
        assert!(entry_names.contains(&"h3-1-aanduiding.pdf".to_string()));
        assert!(entry_names.contains(&"h4-ondersteuningsverklaring.pdf".to_string()));
        assert_eq!(
            entry_names
                .iter()
                .filter(|name| name.starts_with("h9-instemmingsverklaringen/"))
                .count(),
            2
        );
        assert!(
            entry_names
                .iter()
                .all(|name| !name.starts_with("documents-")),
            "did not expect a folder prefix for a single list: {entry_names:?}"
        );

        Ok(())
    }

    #[cfg(feature = "embed-typst")]
    #[tokio::test]
    async fn gen_documents_single_list_allows_candidate_warnings() -> Result<(), AppError> {
        use axum::response::IntoResponse;

        let (store, list_ids, context) =
            setup_documents_test_state(1, 2, true, true, ElectionConfig::EK27).await?;
        let list = store.get_candidate_list(list_ids[0])?;

        let mut dutch_candidate = store.get_person(list.candidates[0])?;
        dutch_candidate.address.street_name = None;
        dutch_candidate.address.postal_code = None;
        dutch_candidate.address.locality = None;
        dutch_candidate.personal_data.bsn = None;
        dutch_candidate.update(&store).await?;

        let mut international_candidate = store.get_person(list.candidates[1])?;
        international_candidate.personal_data.country = Some("BE".parse::<CountryCode>().unwrap());
        international_candidate.personal_data.bsn = Some(BsnOrNoneConfirmed::NoneConfirmed);
        international_candidate.representative = Some(Representative::default());
        international_candidate.update(&store).await?;

        let response = gen_documents(
            DownloadDocumentsPath {
                locale: crate::core::ModelLocale::Nl,
            },
            store,
            State(TypstRenderer::embedded(
                crate::utils::embed_typst::pdf_context(),
            )),
            context,
        )
        .await?
        .into_response();

        let entry_names = crate::test_utils::zip_entry_names(response).await;
        assert!(entry_names.contains(&"eml210.eml.xml".to_string()));
        assert!(entry_names.contains(&"h1-kandidatenlijst.pdf".to_string()));
        assert!(entry_names.contains(&"h3-1-aanduiding.pdf".to_string()));
        assert!(entry_names.contains(&"h4-ondersteuningsverklaring.pdf".to_string()));
        assert_eq!(
            entry_names
                .iter()
                .filter(|name| name.starts_with("h9-instemmingsverklaringen/"))
                .count(),
            2
        );

        Ok(())
    }

    #[cfg(feature = "embed-typst")]
    #[tokio::test]
    async fn gen_documents_single_list_allows_general_information_warnings() -> Result<(), AppError>
    {
        use axum::response::IntoResponse;

        let (store, _, context) =
            setup_documents_test_state(1, 1, true, true, ElectionConfig::EK27).await?;

        let mut political_group = store.get_political_group();
        political_group.legal_name = None;
        political_group.update(&store).await?;

        let mut authorised_agent = store.get_authorised_agents().remove(0);
        authorised_agent.name = FullName::default();
        authorised_agent.update(&store).await?;

        let response = gen_documents(
            DownloadDocumentsPath {
                locale: crate::core::ModelLocale::Nl,
            },
            store,
            State(TypstRenderer::embedded(
                crate::utils::embed_typst::pdf_context(),
            )),
            context,
        )
        .await?
        .into_response();

        let entry_names = crate::test_utils::zip_entry_names(response).await;
        assert!(entry_names.contains(&"eml210.eml.xml".to_string()));
        assert!(entry_names.contains(&"h1-kandidatenlijst.pdf".to_string()));
        assert!(entry_names.contains(&"h3-1-aanduiding.pdf".to_string()));
        assert!(entry_names.contains(&"h4-ondersteuningsverklaring.pdf".to_string()));
        assert_eq!(
            entry_names
                .iter()
                .filter(|name| name.starts_with("h9-instemmingsverklaringen/"))
                .count(),
            1
        );

        Ok(())
    }
}
