use askama::Template;
use axum::{
    extract::Request,
    http::StatusCode,
    middleware::Next,
    response::{IntoResponse, Response},
};
use serde::Serialize;
use tracing::{error, warn};

use crate::{
    AppError, Context, HtmlTemplate, Locale, LocaleValues, auth::csrf_guard::CsrfRejection,
    filters, trans,
};

/// Variants of error responses that can be sent to the client
#[derive(Serialize)]
enum ErrorResponseVariant {
    Unauthorised,
    BadRequest,
    InternalServerError,
    ServiceUnavailable,
    NotFound,
}

impl ErrorResponseVariant {
    fn status_code(&self) -> StatusCode {
        match self {
            ErrorResponseVariant::NotFound => StatusCode::NOT_FOUND,
            ErrorResponseVariant::BadRequest => StatusCode::BAD_REQUEST,
            ErrorResponseVariant::Unauthorised => StatusCode::UNAUTHORIZED,
            ErrorResponseVariant::InternalServerError => StatusCode::INTERNAL_SERVER_ERROR,
            ErrorResponseVariant::ServiceUnavailable => StatusCode::SERVICE_UNAVAILABLE,
        }
    }

    fn title(&self) -> &'static str {
        match self {
            ErrorResponseVariant::Unauthorised => "Unauthorised",
            ErrorResponseVariant::BadRequest => "Bad request",
            ErrorResponseVariant::InternalServerError => "Internal server error",
            ErrorResponseVariant::ServiceUnavailable => "Service unavailable",
            ErrorResponseVariant::NotFound => "Not found",
        }
    }
}

/// Struct representing an error response to be sent to the client
#[derive(Serialize)]
pub struct ErrorResponse {
    error: ErrorResponseVariant,
    message: String,
}

#[derive(Template, Clone)]
#[template(path = "pg/common/pages/error.html")]
pub struct ErrorTemplate {
    pub status_code: StatusCode,
    pub title: String,
    message: String,
}

/// Convert ErrorResponse into an HTTP response
impl IntoResponse for ErrorResponse {
    fn into_response(self) -> Response {
        let ErrorResponse { error, message } = self;
        let status_code = error.status_code();

        let error_template = ErrorTemplate {
            status_code,
            title: error.title().to_string(),
            message,
        };

        let mut response = status_code.into_response();
        response.extensions_mut().insert(error_template);
        response
    }
}

#[derive(Template)]
#[template(path = "pg/common/pages/request_error.html")]
struct RequestErrorTemplate {
    title: String,
    message: String,
}

/// Styled CSRF rejection page, rendered directly with the session locale
/// (the rejection short-circuits above `render_error_pages`).
pub(crate) fn csrf_rejection_response(rejection: CsrfRejection, locale: Locale) -> Response {
    let (status_code, title, message) = match rejection {
        CsrfRejection::InvalidToken => (
            StatusCode::BAD_REQUEST,
            trans!("common.request_error.csrf_title", locale),
            trans!("common.request_error.csrf_message", locale),
        ),
        CsrfRejection::BodyTooLarge => (
            StatusCode::PAYLOAD_TOO_LARGE,
            trans!("common.request_error.too_large_title", locale),
            trans!("common.request_error.too_large_message", locale),
        ),
        CsrfRejection::UnreadableBody => (
            StatusCode::BAD_REQUEST,
            trans!("common.request_error.unreadable_title", locale),
            trans!("common.request_error.unreadable_message", locale),
        ),
    };
    warn!(?rejection, "mutating request rejected by CSRF enforcement");

    let mut response = HtmlTemplate(
        RequestErrorTemplate { title, message },
        LocaleValues { locale },
    )
    .into_response();
    *response.status_mut() = status_code;
    response
}

/// Middleware to render error pages based on ErrorTemplate in response extensions
pub async fn render_error_pages(context: Context, request: Request, next: Next) -> Response {
    let mut response = next.run(request).await;

    match response.extensions_mut().remove::<ErrorTemplate>() {
        None => response,
        Some(error_template) => (
            error_template.status_code,
            HtmlTemplate(error_template, context),
        )
            .into_response(),
    }
}

/// Convert AppError into an HTTP response, via the ErrorResponse struct
impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        ErrorResponse::from_app_error(&self).into_response()
    }
}

/// Convert AppError into ErrorResponse, the AppError contains more information
/// that should not be exposed to the client, but should be logged at this point.
impl From<AppError> for ErrorResponse {
    fn from(err: AppError) -> Self {
        ErrorResponse::from_app_error(&err)
    }
}

impl ErrorResponse {
    fn from_app_error(err: &AppError) -> Self {
        // Infrastructure failures (database unreachable, broken schema) become a
        // 503 so clients and proxies can retry, regardless of which variant
        // carried the failure. Everything else maps per-variant in `build`.
        let response = if err.is_infrastructure_failure() {
            Self::service_unavailable()
        } else {
            Self::build(err)
        };
        log_app_error(err, &response);
        response
    }

    /// The temporary-outage response shared by every infrastructure failure.
    fn service_unavailable() -> Self {
        ErrorResponse {
            error: ErrorResponseVariant::ServiceUnavailable,
            message: "The service is temporarily unavailable. Please try again shortly."
                .to_string(),
        }
    }

    fn build(err: &AppError) -> Self {
        use ErrorResponseVariant::*;

        let internal = || {
            (
                InternalServerError,
                "An internal server error occurred.".to_string(),
            )
        };

        let (error, message) = match err {
            AppError::NotFound(msg) => (NotFound, msg.to_string()),
            AppError::GenericNotFound => (NotFound, "Page not found".to_string()),
            AppError::Unauthorised => (
                Unauthorised,
                "You are not authorised to perform this action.".to_string(),
            ),
            AppError::MultipartFormError(_)
            | AppError::MultipartError(_)
            | AppError::FormRejection(_)
            | AppError::PathRejection(_)
            | AppError::JsonRejection(_)
            | AppError::QueryRejection(_)
            | AppError::UserError(_)
            | AppError::TooManyCandidates
            | AppError::AmbiguousHash => (BadRequest, err.to_string()),
            AppError::EmlError(err) => (BadRequest, format!("EML error: {err}")),
            AppError::IncompleteData(err) => (
                BadRequest,
                format!("Missing data when generating PDF: {err}"),
            ),
            #[cfg(feature = "database")]
            AppError::DatabaseError(_) => internal(),
            #[cfg(feature = "embed-typst")]
            AppError::TypstError(_) => internal(),
            AppError::InternalServerError
            | AppError::NoStorageConfigured
            | AppError::IntegrityViolation
            | AppError::MissingEnvVar(_)
            | AppError::ConfigLoadError(_)
            | AppError::TemplateError(_)
            | AppError::UpstreamError(_)
            | AppError::ServerError(_)
            | AppError::EventDecodeError(_)
            | AppError::AuthError(_) => internal(),
        };

        ErrorResponse { error, message }
    }
}

/// Emit a single tracing event for an error response.
fn log_app_error(err: &AppError, response: &ErrorResponse) {
    match response.error {
        ErrorResponseVariant::InternalServerError | ErrorResponseVariant::ServiceUnavailable => {
            error!(error = ?err, "5xx error");
        }
        ErrorResponseVariant::BadRequest
        | ErrorResponseVariant::Unauthorised
        | ErrorResponseVariant::NotFound => {
            if message_is_safe_to_log(err) {
                warn!(error = ?err, "4xx error");
            } else {
                // Debug of inner extractor errors can echo request input, so
                // log only the variant name (the prefix of the Debug output).
                let dbg = format!("{err:?}");
                let kind = dbg.split_once('(').map_or(dbg.as_str(), |(n, _)| n);
                warn!(kind, "4xx error");
            }
        }
    }
}

/// Return `true` when the user-facing `ErrorResponse.message` for this
/// variant is a constant or developer-authored string and can be safely
/// included in the log event. For variants where the message is built from
/// the inner extractor/validation error (which can echo request input),
/// this returns `false`.
fn message_is_safe_to_log(err: &AppError) -> bool {
    matches!(
        err,
        AppError::Unauthorised
            | AppError::GenericNotFound
            | AppError::UserError(_)
            | AppError::NotFound(_)
            | AppError::IncompleteData(_)
    )
}
