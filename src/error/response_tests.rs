use super::response::*;
use crate::{AppError, AppState, Locale, form::ValidationError, test_utils::response_body_string};
use axum::{
    Router,
    body::Body,
    http::{Request, StatusCode},
    middleware,
    routing::get,
};
use tower::ServiceExt;

#[tokio::test]
async fn not_found_renders_template_with_message() {
    let state = AppState::new_for_tests().await;
    let store = crate::AppStore::new_for_test();
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
    let body = response_body_string(response).await;
    assert!(body.contains("Error 404"));
    assert!(body.contains("missing"));
}

#[tokio::test]
async fn validation_error_maps_to_bad_request() {
    let state = AppState::new_for_tests().await;
    let store = crate::AppStore::new_for_test();
    let app = Router::new()
        .route(
            "/",
            get(|| async {
                let errors = vec![("name".to_string(), ValidationError::InvalidValue)];
                AppError::ValidationError(errors)
            }),
        )
        .layer(middleware::from_fn_with_state(state, render_error_pages));
    let mut request = Request::builder().uri("/").body(Body::empty()).unwrap();
    let mut session = crate::Session::new_test_with_locale(Locale::En);
    session.set_stream_id(crate::StreamId::new());
    request.extensions_mut().insert(session);
    request.extensions_mut().insert(store);
    let response = app.oneshot(request).await.expect("response");

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    let body = response_body_string(response).await;
    assert!(body.contains("Validation error"));
}

#[cfg(feature = "database")]
#[tokio::test]
async fn database_error_maps_to_internal_server_error() {
    let state = AppState::new_for_tests().await;
    let store = crate::AppStore::new_for_test();
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
