use super::app_error::*;
use crate::{
    Context, ErrorResponse, Form, HtmlTemplate, error::response::ErrorTemplate,
    form::ValidationError, test_utils,
};
use axum::{
    body::Body,
    extract::{
        FromRequest, Multipart, Path, Request,
        rejection::{JsonRejection, MissingJsonContentType},
    },
    response::IntoResponse,
};

#[test]
fn displays_not_found_message() {
    let err = AppError::NotFound("missing".to_string());
    assert_eq!(err.to_string(), "missing");
}

#[test]
fn displays_missing_env_var() {
    let err = AppError::MissingEnvVar("STORAGE_URL");
    assert_eq!(err.to_string(), "Missing environment variable: STORAGE_URL");
}

#[test]
fn displays_database_error() {
    #[cfg(feature = "database")]
    {
        let err = AppError::DatabaseError(sqlx::Error::RowNotFound);
        assert!(err.to_string().contains("Database error"));
    }
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
        AppError::CsrfTokenInvalid,
        AppError::NotFound("missing".to_string()),
        AppError::from(askama::Error::Fmt),
        AppError::from(multipart_rejection),
        AppError::from(multipart_error),
        form_rejection,
        AppError::from(json_rejection),
        AppError::from(path_rejection),
        AppError::ValidationError(vec![("name".to_string(), ValidationError::InvalidValue)]),
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
