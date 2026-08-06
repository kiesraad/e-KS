use axum::response::IntoResponse;

use crate::{
    AppError, Context, PgStore, finalise::pages::DownloadDocumentsPath,
    models::documents::DocumentData,
};

pub async fn gen_documents(
    path @ DownloadDocumentsPath { locale }: DownloadDocumentsPath,
    store: PgStore,
    context: Context,
) -> Result<impl IntoResponse, AppError> {
    let (bundles, filename) = DocumentData::from_store_and_context(&store, &context, locale)?;

    DocumentData::serve_download(bundles, filename, path.to_string(), &store, &store).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        ElectionConfig,
        core::ModelLocale,
        structs::{
            common::{BsnOrNoneConfirmed, CountryCode, FullName},
            name_authorisations::NameAuthorisationId,
            persons::Representative,
        },
        test_utils::{sample_name_authorisation, setup_documents_test_state},
    };

    #[tokio::test]
    async fn gen_documents_missing_list_submitter_returns_error() -> Result<(), AppError> {
        let (store, _, context) =
            setup_documents_test_state(1, 1, false, true, ElectionConfig::EK27).await?;
        let result = gen_documents(
            DownloadDocumentsPath {
                locale: crate::core::ModelLocale::Nl,
            },
            store,
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
    async fn gen_documents_multiple_name_authorisations_return_error() -> Result<(), AppError> {
        let (store, _, context) =
            setup_documents_test_state(1, 1, true, true, ElectionConfig::EK27).await?;
        sample_name_authorisation(NameAuthorisationId::new())
            .create(&store)
            .await?;
        let result = gen_documents(
            DownloadDocumentsPath {
                locale: crate::core::ModelLocale::Nl,
            },
            store,
            context,
        )
        .await;

        match result {
            Err(AppError::IncompleteData(message)) => {
                assert_eq!(message, "Expected no more than 1 name authorisation")
            }
            _ => panic!("expected IncompleteData error"),
        }

        Ok(())
    }

    #[tokio::test]
    async fn multiple_name_authorisations_ok_for_list_combinations() -> Result<(), AppError> {
        use axum::response::IntoResponse;

        let (store, _, context) =
            setup_documents_test_state(1, 1, true, true, ElectionConfig::EK27).await?;

        let mut political_group = store.get_political_group();
        political_group.list_designation =
            Some(crate::structs::list_designation::ListDesignation::Combined);
        political_group.update(&store).await?;

        let response = gen_documents(
            DownloadDocumentsPath {
                locale: crate::core::ModelLocale::Nl,
            },
            store,
            context,
        )
        .await?
        .into_response();

        let entry_names = crate::test_utils::zip_entry_names(response).await;
        assert!(entry_names.contains(&"h3-2-samengevoegde-aanduiding.pdf".to_string()));

        Ok(())
    }

    #[tokio::test]
    async fn blank_lists_produce_no_h3() -> Result<(), AppError> {
        use axum::response::IntoResponse;

        let (store, _, context) =
            setup_documents_test_state(1, 1, true, true, ElectionConfig::EK27).await?;

        let mut political_group = store.get_political_group();
        political_group.list_designation =
            Some(crate::structs::list_designation::ListDesignation::Blank);
        political_group.update(&store).await?;

        let response = gen_documents(
            DownloadDocumentsPath {
                locale: crate::core::ModelLocale::Nl,
            },
            store,
            context,
        )
        .await?
        .into_response();

        let entry_names = crate::test_utils::zip_entry_names(response).await;
        assert!(!entry_names.iter().any(|name| name.starts_with("h3")));

        Ok(())
    }

    #[tokio::test]
    async fn gen_documents_missing_designation_returns_error() -> Result<(), AppError> {
        let (store, _, context) =
            setup_documents_test_state(1, 1, true, true, ElectionConfig::EK27).await?;

        let mut political_group = store.get_political_group();
        political_group.display_name = None;
        political_group.update(&store).await?;

        let result = gen_documents(
            DownloadDocumentsPath {
                locale: crate::core::ModelLocale::Nl,
            },
            store,
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

        let result = gen_documents(
            DownloadDocumentsPath {
                locale: ModelLocale::Fry,
            },
            store,
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

    #[tokio::test]
    async fn gen_documents_single_list_allows_general_information_warnings() -> Result<(), AppError>
    {
        use axum::response::IntoResponse;

        let (store, _, context) =
            setup_documents_test_state(1, 1, true, true, ElectionConfig::EK27).await?;

        let mut name_auth = store.get_name_authorisations().remove(0);
        name_auth.name = FullName::default();
        name_auth.legal_name = Default::default();
        name_auth.update(&store).await?;

        let response = gen_documents(
            DownloadDocumentsPath {
                locale: crate::core::ModelLocale::Nl,
            },
            store,
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
