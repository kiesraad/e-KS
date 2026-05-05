use axum::{body::Body, extract::State, http::HeaderValue, response::IntoResponse};
use tokio::io::duplex;
use tokio_util::io::ReaderStream;
use tracing::error;

use crate::{
    AppError, AppEvent, AppStore, Context, TypstRenderer,
    core::ZipResponseWriter,
    submit::{DocumentData, pages::DownloadDocumentsPath},
    utils::no_cache_headers,
};

pub async fn gen_documents(
    path @ DownloadDocumentsPath { list_id, locale }: DownloadDocumentsPath,
    store: AppStore,
    State(renderer): State<TypstRenderer>,
    context: Context,
) -> Result<impl IntoResponse, AppError> {
    let bundle = DocumentData::new(&store, &context, list_id, locale)?;

    store
        .update(AppEvent::DownloadFile {
            file_name: bundle.filename.clone(),
            download_path: path.to_string(),
            list_id,
        })
        .await?;

    let headers = no_cache_headers::generate_attachment_headers(
        &bundle.filename,
        HeaderValue::from_static("application/zip"),
    )?;

    let (reader, writer) = duplex(64 * 1024);
    let body = Body::from_stream(ReaderStream::new(reader));

    tokio::spawn(async move {
        if let Err(err) = bundle
            .write_zip(renderer, ZipResponseWriter::new(writer))
            .await
        {
            error!(error = ?err, "failed to stream submit documents zip");
        }
    });

    Ok((headers, body).into_response())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        authorised_agents::AuthorisedAgentId,
        candidate_lists::CandidateListId,
        list_submitters::ListSubmitterId,
        persons::PersonId,
        test_utils::{
            sample_authorised_agent, sample_candidate_list, sample_list_submitter, sample_person,
        },
    };
    use axum::extract::State;

    #[cfg(feature = "embed-typst")]
    async fn response_body(response: axum::response::Response) -> bytes::Bytes {
        use http_body_util::BodyExt;

        response
            .into_body()
            .collect()
            .await
            .expect("collect body")
            .to_bytes()
    }

    #[cfg(feature = "embed-typst")]
    async fn zip_entry_names(response: axum::response::Response) -> Vec<String> {
        use async_zip::base::read::mem::ZipFileReader;

        let zip = ZipFileReader::new(response_body(response).await.to_vec())
            .await
            .expect("zip body");

        zip.file()
            .entries()
            .iter()
            .map(|entry| {
                entry
                    .filename()
                    .as_str()
                    .expect("utf-8 zip entry name")
                    .to_string()
            })
            .collect()
    }

    async fn setup_documents_test_state(
        candidate_count: usize,
        include_list_submitter: bool,
        include_authorised_agent: bool,
    ) -> Result<(AppStore, CandidateListId, Context), AppError> {
        let store = AppStore::new_for_test();
        let list_id = CandidateListId::new();
        let mut list = sample_candidate_list(list_id);

        if include_list_submitter {
            sample_list_submitter(ListSubmitterId::new())
                .update(&store)
                .await?;
        }

        if include_authorised_agent {
            sample_authorised_agent(AuthorisedAgentId::new())
                .create(&store)
                .await?;
        }

        for _ in 0..candidate_count {
            let person_id = PersonId::new();
            sample_person(person_id).create(&store).await?;
            list.candidates.push(person_id);
        }

        list.create(&store).await?;

        Ok((store, list_id, Context::new_test_without_db()))
    }

    #[tokio::test]
    async fn gen_documents_missing_list_submitter_returns_error() -> Result<(), AppError> {
        let (store, list_id, context) = setup_documents_test_state(1, false, true).await?;
        let renderer = TypstRenderer::http("http://unused.test".to_string());

        let result = gen_documents(
            DownloadDocumentsPath {
                list_id,
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
        let (store, list_id, context) = setup_documents_test_state(1, true, true).await?;
        sample_authorised_agent(AuthorisedAgentId::new())
            .create(&store)
            .await?;
        let renderer = TypstRenderer::http("http://unused.test".to_string());

        let result = gen_documents(
            DownloadDocumentsPath {
                list_id,
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
        let (store, list_id, context) = setup_documents_test_state(1, true, true).await?;

        let mut political_group = store.get_political_group();
        political_group.display_name = None;
        political_group.legal_name = None;
        political_group.update(&store).await?;

        let renderer = TypstRenderer::http("http://unused.test".to_string());

        let result = gen_documents(
            DownloadDocumentsPath {
                list_id,
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

    #[cfg(feature = "embed-typst")]
    #[tokio::test]
    async fn gen_documents_returns_zip_response() -> Result<(), AppError> {
        use axum::{
            http::{StatusCode, header},
            response::IntoResponse,
        };
        use regex::Regex;

        let (store, list_id, context) = setup_documents_test_state(2, true, true).await?;
        let response = gen_documents(
            DownloadDocumentsPath {
                list_id,
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
            Regex::new("attachment; filename=\"documents-(.{2}-)*(.{2})-nl\\.zip\"")
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

        let entry_names = zip_entry_names(response).await;
        assert!(entry_names.contains(&"eml210.eml.xml".to_string()));
        assert!(entry_names.iter().any(|name| name.starts_with("model-h1")));
        assert!(
            entry_names
                .iter()
                .any(|name| name.starts_with("model-h3-1"))
        );
        assert!(entry_names.iter().any(|name| name.starts_with("model-h4")));
        assert_eq!(
            entry_names
                .iter()
                .filter(|name| name.starts_with("model-h9/"))
                .count(),
            2
        );

        Ok(())
    }
}
