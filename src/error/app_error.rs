use axum::extract::{
    multipart::{MultipartError, MultipartRejection},
    rejection::{JsonRejection, PathRejection, QueryRejection},
};
use axum_extra::extract::FormRejection;
use std::{
    convert::Infallible,
    fmt::{Display, Formatter},
};

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
    CsrfTokenInvalid,
    NotFound(String),
    UserError(String),
    #[cfg(feature = "database")]
    DatabaseError(sqlx::Error),
    #[cfg(feature = "embed-typst")]
    TypstError(typst_webservice::AppError),
    TemplateError(askama::Error),
    FormRejection(FormRejection),

    // Axum error types
    MultipartFormError(MultipartError),
    MultipartError(MultipartRejection),
    JsonRejection(JsonRejection),
    PathRejection(PathRejection),
    QueryRejection(QueryRejection),

    // Application level errors
    MissingEnvVar(&'static str),
    ConfigLoadError(String),
    ServerError(std::io::Error),
    UpstreamError(reqwest::Error),

    /// Missing data when generating a PDF.
    IncompleteData(&'static str),

    EmlError(eml_nl::EMLError),

    AuthError(auth_service::error::AuthError),

    NoStorageConfigured,
    IntegrityViolation,

    /// Attempted to add a candidate to a list that is already at the maximum
    /// allowed number of candidates ([`crate::MAX_CANDIDATES`]).
    TooManyCandidates,

    /// A hash prefix matched more than one event; the user must supply a longer prefix.
    AmbiguousHash,

    /// A persisted event could not be decrypted or deserialized.
    /// Indicates tampering, a wrong key, or a corrupt/unsupported frame.
    EventDecodeError(String),
}

impl Display for AppError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            AppError::ConfigLoadError(err) => write!(f, "Configuration load error: {err}"),
            AppError::CsrfTokenInvalid => write!(f, "CSRF token is invalid"),
            #[cfg(feature = "database")]
            AppError::DatabaseError(err) => write!(f, "Database error: {err}"),
            #[cfg(feature = "embed-typst")]
            AppError::TypstError(err) => write!(f, "Typst error: {err}"),
            AppError::FormRejection(err) => write!(f, "Form error: {err}"),
            AppError::GenericNotFound => write!(f, "Page not found"),
            AppError::IntegrityViolation => write!(f, "Data integrity violation"),
            AppError::TooManyCandidates => write!(
                f,
                "Cannot add more than {} candidates to a candidate list",
                crate::MAX_CANDIDATES
            ),
            AppError::AmbiguousHash => write!(f, "Ambiguous hash prefix"),
            AppError::InternalServerError => write!(f, "Internal server error"),
            AppError::JsonRejection(err) => write!(f, "JSON error: {err}"),
            AppError::MissingEnvVar(var) => write!(f, "Missing environment variable: {var}"),
            AppError::MultipartError(err) => write!(f, "Multipart error: {err}"),
            AppError::MultipartFormError(err) => write!(f, "Multipart form error: {err}"),
            AppError::NoStorageConfigured => write!(f, "No event storage configured"),
            AppError::NotFound(msg) => write!(f, "{msg}"),
            AppError::UserError(msg) => write!(f, "{msg}"),
            AppError::PathRejection(err) => write!(f, "Path error: {err}"),
            AppError::QueryRejection(err) => write!(f, "Query error: {err}"),
            AppError::ServerError(err) => write!(f, "Server error: {err}"),
            AppError::TemplateError(err) => write!(f, "Template error: {err}"),
            AppError::Unauthorised => write!(f, "Unauthorised"),
            AppError::UpstreamError(err) => write!(f, "Upstream error: {err}"),
            AppError::IncompleteData(err) => write!(f, "Missing data when generating PDF: {err}"),
            AppError::EventDecodeError(err) => write!(f, "Event decode error: {err}"),
            AppError::EmlError(err) => write!(f, "EML error: {err}"),
            AppError::AuthError(err) => write!(f, "Authentication error: {err}"),
        }
    }
}

impl std::error::Error for AppError {}

#[cfg(feature = "database")]
impl From<sqlx::Error> for AppError {
    fn from(err: sqlx::Error) -> Self {
        AppError::DatabaseError(err)
    }
}

#[cfg(feature = "embed-typst")]
impl From<typst_webservice::AppError> for AppError {
    fn from(err: typst_webservice::AppError) -> Self {
        AppError::TypstError(err)
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

impl From<reqwest::Error> for AppError {
    fn from(err: reqwest::Error) -> Self {
        Self::UpstreamError(err)
    }
}

impl From<serde_json::Error> for AppError {
    fn from(_: serde_json::Error) -> Self {
        AppError::InternalServerError
    }
}

impl From<csv::Error> for AppError {
    fn from(_: csv::Error) -> Self {
        AppError::InternalServerError
    }
}

impl From<eml_nl::EMLError> for AppError {
    fn from(err: eml_nl::EMLError) -> Self {
        AppError::EmlError(err)
    }
}

impl From<auth_service::error::AuthError> for AppError {
    fn from(err: auth_service::error::AuthError) -> Self {
        AppError::AuthError(err)
    }
}
#[cfg(test)]
mod tests {
    use crate::AppError;

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
}
