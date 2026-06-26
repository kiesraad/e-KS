use async_zip::base::read::mem::ZipFileReader;
use axum::{
    Router,
    body::Body,
    http::{Request, StatusCode, header},
};
use http_body_util::BodyExt;
use tower::ServiceExt;
use tracing_test::traced_test;

use crate::{
    AppError, AppEvent, AppState, AppStore, Config, ElectionConfig, Locale, Session, StreamId,
    candidate_lists::CandidateListId,
    common::PreviousElectionResults,
    core::ModelLocale,
    list_submitters::ListSubmitterId,
    name_authorisations::NameAuthorisationId,
    persons::PersonId,
    store::StoreEvent,
    test_utils::{
        sample_candidate_list, sample_list_submitter, sample_name_authorisation, sample_person,
        sample_political_group,
    },
};

use super::DownloadDocumentsPath;

async fn setup_app() -> Result<(Router, AppStore, Session), AppError> {
    let config = Config::new_test();

    let state = AppState::new_for_tests_with_config(config).await;
    let stream_id = StreamId::new();
    let store = state
        .store_for_stream(stream_id, ElectionConfig::EK27, false)
        .await?;
    sample_political_group().update(&store).await?;

    let mut session = Session::new_test_with_locale(Locale::En);
    session.set_stream_id(stream_id);

    Ok((super::router().with_state(state), store, session))
}

struct DownloadTestState {
    app: Router,
    store: AppStore,
    session: Session,
}

async fn setup_download_test_state(
    list_count: usize,
    candidate_count: usize,
    include_list_submitter: bool,
) -> Result<DownloadTestState, AppError> {
    let (app, store, session) = setup_app().await?;
    if include_list_submitter {
        sample_list_submitter(ListSubmitterId::new())
            .update(&store)
            .await?;
    }
    if store.get_name_authorisations().is_empty() {
        sample_name_authorisation(NameAuthorisationId::new())
            .create(&store)
            .await?;
    }

    for _ in 0..list_count {
        let list_id = CandidateListId::new();
        let mut list = sample_candidate_list(list_id);
        list.create(&store).await?;

        for _ in 0..candidate_count {
            let person_id = PersonId::new();
            sample_person(person_id).create(&store).await?;
            list.append_candidate(&store, person_id).await?;
        }
    }

    Ok(DownloadTestState {
        app,
        store,
        session,
    })
}

async fn download_file(
    download_path: &str,
    include_list_submitter: bool,
) -> Result<Vec<(String, String)>, AppError> {
    let DownloadTestState {
        app,
        store,
        session,
    } = setup_download_test_state(2, 1, include_list_submitter).await?;

    app.oneshot(request(download_path.to_string(), session, store.clone()))
        .await
        .unwrap();

    Ok(store
        .get_events()
        .into_iter()
        .filter_map(|event| match event {
            StoreEvent {
                payload:
                    AppEvent::DownloadFile {
                        file_name,
                        download_path,
                    },
                ..
            } => Some((file_name, download_path)),
            _ => None,
        })
        .collect())
}

fn request(uri: String, session: Session, store: AppStore) -> Request<Body> {
    let mut request = Request::builder().uri(uri).body(Body::empty()).unwrap();
    request.extensions_mut().insert(session);
    request.extensions_mut().insert(store);
    request
}

async fn body_bytes(response: axum::response::Response) -> bytes::Bytes {
    response
        .into_body()
        .collect()
        .await
        .expect("collect body")
        .to_bytes()
}

async fn zip_entry_names(response: axum::response::Response) -> Vec<String> {
    let body = body_bytes(response).await.to_vec();
    let zip = ZipFileReader::new(body).await.expect("zip body");
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

#[tokio::test]
#[traced_test]
async fn download_documents_endpoint_returns_zip() -> Result<(), AppError> {
    let DownloadTestState {
        app,
        store,
        session,
    } = setup_download_test_state(1, 2, true).await?;

    let response = app
        .oneshot(request(
            DownloadDocumentsPath {
                locale: ModelLocale::Nl,
            }
            .to_string(),
            session,
            store.clone(),
        ))
        .await
        .expect("submit documents response");

    let status = response.status();
    let headers = response.headers().clone();
    let entry_names = zip_entry_names(response).await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(
        headers
            .get(header::CONTENT_TYPE)
            .expect("content type header"),
        "application/zip"
    );
    let content_disposition = headers
        .get(header::CONTENT_DISPOSITION)
        .and_then(|value| value.to_str().ok())
        .expect("content disposition header");
    assert!(
        regex::Regex::new("^attachment; filename=\"kiesraad-demo-ek27-v\\d+\\.zip\"$")
            .unwrap()
            .is_match(content_disposition),
        "unexpected content disposition: {content_disposition}",
    );
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
#[traced_test]
async fn download_documents_excludes_h4_when_previously_seated() -> Result<(), AppError> {
    let DownloadTestState {
        app,
        store,
        session,
    } = setup_download_test_state(1, 2, true).await?;

    // Update political group to previously seated
    let mut group = sample_political_group();
    group.previous_election_results = Some(PreviousElectionResults::OneToFifteenSeats);
    group.update(&store).await?;

    let response = app
        .oneshot(request(
            DownloadDocumentsPath {
                locale: ModelLocale::Nl,
            }
            .to_string(),
            session,
            store.clone(),
        ))
        .await
        .expect("submit documents response");

    let entry_names = zip_entry_names(response).await;

    assert!(entry_names.contains(&"h1-kandidatenlijst.pdf".to_string()));
    assert!(!entry_names.iter().any(|e| e.starts_with("h4")));

    Ok(())
}

#[tokio::test]
#[traced_test]
async fn documents_download_adds_download_event() -> Result<(), AppError> {
    let download_path = DownloadDocumentsPath {
        locale: ModelLocale::Nl,
    }
    .to_string();

    let events = download_file(&download_path, true).await?;

    assert_eq!(events.len(), 1);
    let filename_pattern = regex::Regex::new("^kiesraad-demo-ek27-v\\d+\\.zip$").unwrap();
    for (file_name, actual_download_path) in events {
        assert!(
            filename_pattern.is_match(&file_name),
            "unexpected file_name: {file_name}",
        );
        assert_eq!(download_path, actual_download_path);
    }

    Ok(())
}
