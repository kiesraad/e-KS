use axum::extract::{
    multipart::MultipartError,
    rejection::{FormRejection, JsonRejection, PathRejection},
};
use std::fmt::{Display, Formatter};

use crate::form::FieldErrors;

/// Type alias for application responses
pub type AppResponse<T> = Result<T, AppError>;

/// Application wide error enum
#[derive(Default, Debug)]
pub enum AppError {
    // Request level errors
    Unauthorized,
    InternalServerError,
    #[default]
    GenericNotFound,
    NotFound(String),
    DatabaseError(sqlx::Error),
    TemplateError(askama::Error),
    MultipartFormError(MultipartError),
    FormRejection(FormRejection),
    ValidationError(FieldErrors),
    JsonRejection(JsonRejection),
    PathRejection(PathRejection),

    // Application level errors
    MissingEnvVar(&'static str),
    ConfigLoadError(String),
    ServerError(std::io::Error),
}

impl Display for AppError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            AppError::Unauthorized => write!(f, "Unauthorized"),
            AppError::InternalServerError => write!(f, "Internal server error"),
            AppError::DatabaseError(err) => write!(f, "Database error: {err}"),
            AppError::TemplateError(err) => write!(f, "Template error: {err}"),
            AppError::MissingEnvVar(var) => write!(f, "Missing environment variable: {var}"),
            AppError::ConfigLoadError(err) => write!(f, "Configuration load error: {err}"),
            AppError::ServerError(err) => write!(f, "Server error: {err}"),
            AppError::MultipartFormError(err) => write!(f, "Multipart form error: {err}"),
            AppError::FormRejection(err) => write!(f, "Form error: {err}"),
            AppError::PathRejection(err) => write!(f, "Path error: {err}"),
            AppError::ValidationError(errors) => write!(f, "Validation error: {errors:?}"),
            AppError::JsonRejection(err) => write!(f, "JSON error: {err}"),
            AppError::NotFound(msg) => write!(f, "{msg}"),
            AppError::GenericNotFound => write!(f, "Page not found"),
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        Context, ErrorResponse, HtmlTemplate, Locale, error::response::ErrorTemplate,
        form::ValidationError, test_utils,
    };
    use axum::{
        body::Body,
        extract::{FromRequest, Multipart},
        http::{Request, header::CONTENT_TYPE},
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

    async fn sample_multipart_error() -> MultipartError {
        let boundary = "test-boundary";
        let body = format!("--{boundary}\r\n");
        let request = Request::builder()
            .header(
                CONTENT_TYPE,
                format!("multipart/form-data; boundary={boundary}"),
            )
            .body(Body::from(body))
            .expect("request");
        let mut multipart = Multipart::from_request(request, &())
            .await
            .expect("multipart");

        multipart.next_field().await.expect_err("multipart error")
    }

    #[tokio::test]
    async fn app_error_variants_convert_to_error_response() {
        let multipart_error = sample_multipart_error().await;

        let errors = vec![
            AppError::Unauthorized,
            AppError::InternalServerError,
            AppError::GenericNotFound,
            AppError::NotFound("missing".to_string()),
            AppError::DatabaseError(sqlx::Error::RowNotFound),
            AppError::TemplateError(askama::Error::Fmt),
            AppError::MultipartFormError(multipart_error),
            AppError::ValidationError(vec![("name".to_string(), ValidationError::InvalidValue)]),
            AppError::MissingEnvVar("DATABASE_URL"),
            AppError::ConfigLoadError("bad".to_string()),
            AppError::ServerError(std::io::Error::other("oh nooo")),
        ];

        for error in errors {
            let error_response = ErrorResponse::from(error);
            let response = error_response.into_response();
            let error_template = response.extensions().get::<ErrorTemplate>().unwrap();
            let content = error_template.title.clone();
            let context = Context::new(Locale::En);
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
