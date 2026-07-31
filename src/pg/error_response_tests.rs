use super::error_response::*;
use crate::{AppError, AppState, Context, Form, HtmlTemplate, Locale, test_utils};
use axum::{
    Router,
    body::Body,
    extract::{
        FromRequest, Multipart, Path, Request,
        rejection::{JsonRejection, MissingJsonContentType},
    },
    http::StatusCode,
    middleware,
    response::IntoResponse,
    routing::get,
};
use tower::ServiceExt;

#[tokio::test]
async fn not_found_renders_template_with_message() {
    let state = AppState::new_for_tests().await;
    let store = crate::PgStore::new_for_test();
    let app = Router::new()
        .route(
            "/",
            get(|| async { AppError::NotFound("missing".to_string()) }),
        )
        .layer(middleware::from_fn_with_state(state, render_error_pages));

    let mut request = Request::builder().uri("/").body(Body::empty()).unwrap();
    let mut session = crate::Session::new_test_with_locale(Locale::En);
    session.set_stream_id(crate::StreamId::new());
    request.extensions_mut().insert(session);
    request.extensions_mut().insert(store);
    let response = app.oneshot(request).await.expect("response");

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
    let body = test_utils::response_body_string(response).await;
    assert!(body.contains("Error 404"));
    assert!(body.contains("missing"));
}

#[cfg(feature = "database")]
#[tokio::test]
async fn database_error_maps_to_internal_server_error() {
    let state = AppState::new_for_tests().await;
    let store = crate::PgStore::new_for_test();
    let app = Router::new()
        .route(
            "/",
            get(|| async { AppError::DatabaseError(sqlx::Error::RowNotFound) }),
        )
        .layer(middleware::from_fn_with_state(state, render_error_pages));
    let mut request = Request::builder().uri("/").body(Body::empty()).unwrap();
    let mut session = crate::Session::new_test_with_locale(Locale::En);
    session.set_stream_id(crate::StreamId::new());
    request.extensions_mut().insert(session);
    request.extensions_mut().insert(store);
    let response = app.oneshot(request).await.expect("response");

    assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
}

#[cfg(feature = "acme")]
#[tokio::test]
async fn acme_error_maps_to_internal_server_error() {
    let state = AppState::new_for_tests().await;
    let store = crate::PgStore::new_for_test();
    let app = Router::new()
        .route(
            "/",
            get(|| async { AppError::AcmeError(instant_acme::Error::Str("boom")) }),
        )
        .layer(middleware::from_fn_with_state(state, render_error_pages));
    let mut request = Request::builder().uri("/").body(Body::empty()).unwrap();
    let mut session = crate::Session::new_test_with_locale(Locale::En);
    session.set_stream_id(crate::StreamId::new());
    request.extensions_mut().insert(session);
    request.extensions_mut().insert(store);
    let response = app.oneshot(request).await.expect("response");

    assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
}

fn get_multipart_error_request() -> Request<Body> {
    let body = "--boundary\r\n\
            Content-Disposition: form-data; name=\"fiel";

    Request::builder()
        .method("POST")
        .uri("/upload")
        .header("Content-Type", "multipart/form-data; boundary=boundary")
        .body(Body::from(body))
        .unwrap()
}

fn get_multipart_rejection_request() -> Request<Body> {
    Request::builder()
        .method("POST")
        .uri("/upload")
        .body(Body::from("not multipart"))
        .unwrap()
}

#[tokio::test]
async fn app_error_variants_convert_to_error_response() {
    let form_rejection = Form::<bool>::from_request(
        Request::builder()
            .uri("/save")
            .body(Body::from("incorrect"))
            .unwrap(),
        &(),
    )
    .await
    .unwrap_err();
    let json_rejection: JsonRejection = MissingJsonContentType::default().into();
    let multipart_rejection = Multipart::from_request(get_multipart_rejection_request(), &())
        .await
        .unwrap_err();
    let mut multipart_form_result = Multipart::from_request(get_multipart_error_request(), &())
        .await
        .unwrap();
    let multipart_error = multipart_form_result.next_field().await.unwrap_err();
    let path_rejection = Path::<i32>::from_request(
        Request::builder()
            .uri("/not-a-number")
            .body(Body::empty())
            .unwrap(),
        &(),
    )
    .await
    .unwrap_err();

    let errors = vec![
        AppError::Unauthorised,
        AppError::InternalServerError,
        AppError::GenericNotFound,
        AppError::NotFound("missing".to_string()),
        AppError::from(askama::Error::Fmt),
        AppError::from(multipart_rejection),
        AppError::from(multipart_error),
        form_rejection,
        AppError::from(json_rejection),
        AppError::from(path_rejection),
        AppError::MissingEnvVar("STORAGE_URL"),
        AppError::ConfigLoadError("bad".to_string()),
        AppError::ServerError(std::io::Error::other("oh nooo")),
        #[cfg(feature = "database")]
        AppError::from(sqlx::Error::RowNotFound),
    ];

    for error in errors {
        let message = error.to_string();

        assert!(!message.is_empty());

        let error_response = ErrorResponse::from(error);
        let response = error_response.into_response();
        let error_template = response.extensions().get::<ErrorTemplate>().unwrap();
        let content = error_template.title.clone();
        let context = Context::new_test_without_db();
        let html_response = (
            error_template.status_code,
            HtmlTemplate(error_template, context),
        )
            .into_response();

        assert_eq!(html_response.status(), response.status());

        let body = test_utils::response_body_string(html_response).await;

        assert!(body.contains(&content));
    }
}
