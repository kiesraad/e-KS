use axum::extract::{
    multipart::{MultipartError, MultipartRejection},
    rejection::{FormRejection, JsonRejection, PathRejection, QueryRejection},
};
use std::{
    convert::Infallible,
    fmt::{Display, Formatter},
};

use crate::form::FieldErrors;

/// Type alias for application responses
pub type AppResponse<T> = Result<T, AppError>;

/// Application wide error enum
#[derive(Default, Debug)]
pub enum AppError {
    // Request level errors
    Unauthorised,
    InternalServerError,
    #[default]
    GenericNotFound,
    NotFound(String),
    DatabaseError(sqlx::Error),
    TemplateError(askama::Error),
    FormRejection(FormRejection),
    // Axum error types
    MultipartFormError(MultipartError),
    MultipartError(MultipartRejection),
    ValidationError(FieldErrors),
    JsonRejection(JsonRejection),
    PathRejection(PathRejection),
    QueryRejection(QueryRejection),

    // Application level errors
    MissingEnvVar(&'static str),
    ConfigLoadError(String),
    ServerError(std::io::Error),

    NoStorageConfigured,
    IntegrityViolation,
}

impl Display for AppError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            AppError::Unauthorised => write!(f, "Unauthorised"),
            AppError::InternalServerError => write!(f, "Internal server error"),
            AppError::DatabaseError(err) => write!(f, "Database error: {err}"),
            AppError::TemplateError(err) => write!(f, "Template error: {err}"),
            AppError::MissingEnvVar(var) => write!(f, "Missing environment variable: {var}"),
            AppError::ConfigLoadError(err) => write!(f, "Configuration load error: {err}"),
            AppError::ServerError(err) => write!(f, "Server error: {err}"),
            AppError::MultipartFormError(err) => write!(f, "Multipart form error: {err}"),
            AppError::MultipartError(err) => write!(f, "Multipart error: {err}"),
            AppError::FormRejection(err) => write!(f, "Form error: {err}"),
            AppError::PathRejection(err) => write!(f, "Path error: {err}"),
            AppError::ValidationError(errors) => write!(f, "Validation error: {errors:?}"),
            AppError::JsonRejection(err) => write!(f, "JSON error: {err}"),
            AppError::QueryRejection(err) => write!(f, "Query error: {err}"),
            AppError::NotFound(msg) => write!(f, "{msg}"),
            AppError::GenericNotFound => write!(f, "Page not found"),
            AppError::NoStorageConfigured => write!(f, "No event storage configured"),
            AppError::IntegrityViolation => write!(f, "Data integrity violation"),
        }
    }
}

impl std::error::Error for AppError {}

impl From<sqlx::Error> for AppError {
    fn from(err: sqlx::Error) -> Self {
        AppError::DatabaseError(err)
    }
}

impl From<std::fmt::Error> for AppError {
    fn from(_: std::fmt::Error) -> Self {
        AppError::InternalServerError
    }
}

impl From<askama::Error> for AppError {
    fn from(err: askama::Error) -> Self {
        AppError::TemplateError(err)
    }
}

impl From<MultipartError> for AppError {
    fn from(err: MultipartError) -> Self {
        AppError::MultipartFormError(err)
    }
}

impl From<MultipartRejection> for AppError {
    fn from(err: MultipartRejection) -> Self {
        AppError::MultipartError(err)
    }
}

impl From<FormRejection> for AppError {
    fn from(err: FormRejection) -> Self {
        AppError::FormRejection(err)
    }
}

impl From<JsonRejection> for AppError {
    fn from(err: JsonRejection) -> Self {
        AppError::JsonRejection(err)
    }
}

impl From<PathRejection> for AppError {
    fn from(err: PathRejection) -> Self {
        AppError::PathRejection(err)
    }
}

impl From<QueryRejection> for AppError {
    fn from(err: QueryRejection) -> Self {
        AppError::QueryRejection(err)
    }
}

impl From<Infallible> for AppError {
    fn from(_: Infallible) -> Self {
        AppError::InternalServerError
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        Context, ErrorResponse, HtmlTemplate, error::response::ErrorTemplate,
        form::ValidationError, test_utils,
    };
    use axum::{
        body::Body,
        extract::{
            FromRequest, Multipart, Path, Request,
            rejection::{InvalidFormContentType, JsonRejection, MissingJsonContentType},
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
        let err = AppError::MissingEnvVar("DATABASE_URL");
        assert_eq!(
            err.to_string(),
            "Missing environment variable: DATABASE_URL"
        );
    }

    #[test]
    fn displays_database_error() {
        let err = AppError::DatabaseError(sqlx::Error::RowNotFound);
        assert!(err.to_string().contains("Database error"));
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
        let form_rejection: FormRejection = InvalidFormContentType::default().into();
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
            AppError::from(sqlx::Error::RowNotFound),
            AppError::from(askama::Error::Fmt),
            AppError::from(multipart_rejection),
            AppError::from(multipart_error),
            AppError::from(form_rejection),
            AppError::from(json_rejection),
            AppError::from(path_rejection),
            AppError::ValidationError(vec![("name".to_string(), ValidationError::InvalidValue)]),
            AppError::MissingEnvVar("DATABASE_URL"),
            AppError::ConfigLoadError("bad".to_string()),
            AppError::ServerError(std::io::Error::other("oh nooo")),
        ];

        for error in errors {
            let message = error.to_string();

            assert!(!message.is_empty());

            let error_response = ErrorResponse::from(error);
            let response = error_response.into_response();
            let error_template = response.extensions().get::<ErrorTemplate>().unwrap();
            let content = error_template.title.clone();
            let context = Context::new_test().await;
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
}
