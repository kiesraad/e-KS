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
}
