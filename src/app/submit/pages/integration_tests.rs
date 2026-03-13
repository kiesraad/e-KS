use axum::{
    Router,
    body::Body,
    http::{Request, StatusCode, header},
};
use http_body_util::BodyExt;
use reqwest::Client;
use tokio::time::{Duration, sleep};
use tower::ServiceExt;

use crate::{
    AppError, AppState, AppStore, Config, Locale, PoliticalGroupId, Session,
    candidate_lists::CandidateListId,
    core::ModelLocale,
    list_submitters::ListSubmitterId,
    persons::PersonId,
    test_utils::{
        sample_candidate_list, sample_list_submitter, sample_person, sample_political_group,
    },
};

use super::{DownloadH1Path, DownloadH9Path};

async fn typst_url() -> String {
    let url = crate::utils::embed_typst::start()
        .await
        .expect("start embedded typst server");
    let client = Client::new();

    for _ in 0..20 {
        if let Ok(response) = client.get(&url).send().await
            && response.status() == StatusCode::OK
        {
            return url;
        }
        sleep(Duration::from_millis(25)).await;
    }

    panic!("embedded typst server did not return 200 OK on /");
}

async fn setup_app() -> Result<(Router, AppStore, Session), AppError> {
    let mut config = Config::new_test();
    config.typst_url = typst_url().await;

    let state = AppState::new_with_config(config).await?;
    let political_group_id = PoliticalGroupId::new();
    let store = state.store_for_political_group(political_group_id).await?;
    sample_political_group(political_group_id)
        .update(&store)
        .await?;

    let mut session = Session::new_with_locale(Locale::En);
    session.set_political_group(political_group_id);

    Ok((super::router().with_state(state), store, session))
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

async fn assert_download_response(
    response: axum::response::Response,
    content_type: &'static str,
    filename_prefix: &'static str,
    extension: &'static str,
    body_prefix: &'static [u8],
    body_kind: &'static str,
) {
    assert_eq!(response.status(), StatusCode::OK);
    let headers = response.headers();
    assert_eq!(
        headers
            .get(header::CONTENT_TYPE)
            .expect("content type header"),
        content_type
    );
    let disposition = headers
        .get(header::CONTENT_DISPOSITION)
        .and_then(|value| value.to_str().ok())
        .expect("content disposition header");
    assert!(disposition.starts_with(filename_prefix));
    assert!(disposition.ends_with(extension));

    let body = body_bytes(response).await;
    assert!(
        body.as_ref().starts_with(body_prefix),
        "expected {body_kind} body"
    );
}

#[tokio::test]
async fn download_h1_endpoint_returns_pdf() -> Result<(), AppError> {
    let (app, store, session) = setup_app().await?;
    let list_id = CandidateListId::new();
    let list_submitter_id = ListSubmitterId::new();
    let person_id = PersonId::new();

    sample_list_submitter(list_submitter_id)
        .create(&store)
        .await?;
    sample_person(person_id).create(&store).await?;

    let mut list = sample_candidate_list(list_id);
    list.list_submitter_id = Some(list_submitter_id);
    list.create(&store).await?;
    list.append_candidate(&store, person_id).await?;

    let response = app
        .oneshot(request(
            DownloadH1Path {
                list_id,
                locale: ModelLocale::Nl,
            }
            .to_string(),
            session,
            store,
        ))
        .await
        .expect("submit h1 response");

    assert_download_response(
        response,
        "application/pdf",
        "attachment; filename=\"model-h1-nl-(",
        ").pdf\"",
        b"%PDF-",
        "PDF",
    )
    .await;

    Ok(())
}

#[tokio::test]
async fn download_h9_endpoint_returns_zip() -> Result<(), AppError> {
    let (app, store, session) = setup_app().await?;
    let list_id = CandidateListId::new();

    let person_id_1 = PersonId::new();
    sample_person(person_id_1).create(&store).await?;

    let person_id_2 = PersonId::new();
    sample_person(person_id_2).create(&store).await?;

    let mut list = sample_candidate_list(list_id);
    list.create(&store).await?;
    list.append_candidate(&store, person_id_1).await?;
    list.append_candidate(&store, person_id_2).await?;

    let response = app
        .oneshot(request(
            DownloadH9Path {
                list_id,
                locale: ModelLocale::Nl,
            }
            .to_string(),
            session,
            store,
        ))
        .await
        .expect("submit h9 response");

    assert_download_response(
        response,
        "application/zip",
        "attachment; filename=\"model-h9-nl-(",
        ").zip\"",
        b"PK",
        "ZIP",
    )
    .await;

    Ok(())
}
