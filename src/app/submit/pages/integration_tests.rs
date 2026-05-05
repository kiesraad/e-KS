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
    authorised_agents::AuthorisedAgentId,
    candidate_lists::CandidateListId,
    core::ModelLocale,
    list_submitters::ListSubmitterId,
    persons::PersonId,
    store::StoreEvent,
    test_utils::{
        sample_authorised_agent, sample_candidate_list, sample_list_submitter, sample_person,
        sample_political_group,
    },
};

use super::DownloadDocumentsPath;

async fn setup_app() -> Result<(Router, AppStore, Session), AppError> {
    let config = Config::new_test();

    let state = AppState::new_with_config(config).await?;
    let stream_id = StreamId::new();
    let store = state
        .store_for_stream(stream_id, ElectionConfig::EK27, true)
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
    list_id: CandidateListId,
}

async fn setup_download_test_state(
    candidate_count: usize,
    include_list_submitter: bool,
    list_id: Option<CandidateListId>,
) -> Result<DownloadTestState, AppError> {
    let (app, store, session) = setup_app().await?;
    let list_id = list_id.unwrap_or_default();

    let mut list = sample_candidate_list(list_id);
    if include_list_submitter {
        sample_list_submitter(ListSubmitterId::new())
            .update(&store)
            .await?;
    }
    if store.get_authorised_agents().is_empty() {
        sample_authorised_agent(AuthorisedAgentId::new())
            .create(&store)
            .await?;
    }
    list.create(&store).await?;

    for _ in 0..candidate_count {
        let person_id = PersonId::new();
        sample_person(person_id).create(&store).await?;
        list.append_candidate(&store, person_id).await?;
    }

    Ok(DownloadTestState {
        app,
        store,
        session,
        list_id,
    })
}

async fn download_file(
    download_path: &str,
    list_id: CandidateListId,
    include_list_submitter: bool,
) -> Result<(String, String, CandidateListId), AppError> {
    let DownloadTestState {
        app,
        store,
        session,
        list_id: _,
    } = setup_download_test_state(1, include_list_submitter, Some(list_id)).await?;

    app.oneshot(request(download_path.to_string(), session, store.clone()))
        .await
        .unwrap();

    // Consume last event and check that it is a DownloadFile event
    let mut events = store.get_events();
    let Some(StoreEvent {
        payload:
            AppEvent::DownloadFile {
                file_name,
                download_path: actual_download_path,
                list_id: actual_list_id,
            },
        ..
    }) = events.pop()
    else {
        panic!("expected the last event to be a download event")
    };

    Ok((file_name, actual_download_path, actual_list_id))
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
        list_id,
    } = setup_download_test_state(2, true, None).await?;

    let response = app
        .oneshot(request(
            DownloadDocumentsPath {
                list_id,
                locale: ModelLocale::Nl,
            }
            .to_string(),
            session,
            store,
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
    assert!(
        headers
            .get(header::CONTENT_DISPOSITION)
            .and_then(|value| value.to_str().ok())
            .expect("content disposition header")
            .starts_with("attachment; filename=\"documents-")
    );
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
    assert!(entry_names.iter().any(|name| name == "eml210.eml.xml"));

    Ok(())
}

#[tokio::test]
#[traced_test]
async fn documents_download_adds_download_event() -> Result<(), AppError> {
    let list_id = CandidateListId::new();
    let download_path = DownloadDocumentsPath {
        list_id,
        locale: ModelLocale::Nl,
    }
    .to_string();

    let (file_name, actual_download_path, actual_list_id) =
        download_file(&download_path, list_id, true).await?;

    assert_eq!(file_name, "documents-ut.zip");
    assert_eq!(download_path, actual_download_path);
    assert_eq!(list_id, actual_list_id);

    Ok(())
}
