use askama::Template;
use axum::{
    extract::Request,
    http::StatusCode,
    middleware::Next,
    response::{IntoResponse, Response},
};
use serde::Serialize;
use tracing::error;

use crate::{AppError, Context, HtmlTemplate, filters, t};

/// Variants of error responses that can be sent to the client
#[derive(Serialize)]
enum ErrorResponseVariant {
    Unauthorized,
    BadRequest,
    InternalServerError,
    NotFound,
}

impl ErrorResponseVariant {
    fn status_code(&self) -> StatusCode {
        match self {
            ErrorResponseVariant::NotFound => StatusCode::NOT_FOUND,
            ErrorResponseVariant::BadRequest => StatusCode::BAD_REQUEST,
            ErrorResponseVariant::Unauthorized => StatusCode::UNAUTHORIZED,
            ErrorResponseVariant::InternalServerError => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn title(&self) -> &'static str {
        match self {
            ErrorResponseVariant::Unauthorized => "Unauthorized",
            ErrorResponseVariant::BadRequest => "Bad request",
            ErrorResponseVariant::InternalServerError => "Internal server error",
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
#[template(path = "error.html")]
struct ErrorTemplate {
    status_code: StatusCode,
    title: String,
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

/// Middleware to render error pages based on ErrorTemplate in response extensions
pub async fn render_error_pages(context: Context, request: Request, next: Next) -> Response {
    let mut response = next.run(request).await;

    match response.extensions_mut().remove::<ErrorTemplate>() {
        None => response,
        Some(error_template) => (
            error_template.status_code,
            HtmlTemplate(error_template.clone(), context),
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
        error!(?err, "Error while processing request");

        ErrorResponse::from_app_error(&err)
    }
}

impl ErrorResponse {
    fn from_app_error(err: &AppError) -> Self {
        error!(?err);

        match err {
            AppError::NotFound(msg) => ErrorResponse {
                error: ErrorResponseVariant::NotFound,
                message: msg.to_string(),
            },
            AppError::GenericNotFound => ErrorResponse {
                error: ErrorResponseVariant::NotFound,
                message: "Page not found".to_string(),
            },
            AppError::Unauthorized => ErrorResponse {
                error: ErrorResponseVariant::Unauthorized,
                message: "You are not authorized to perform this action.".to_string(),
            },
            AppError::MultipartFormError(e) => ErrorResponse {
                error: ErrorResponseVariant::BadRequest,
                message: format!("Bad request: {e}"),
            },
            AppError::FormRejection(e) => ErrorResponse {
                error: ErrorResponseVariant::BadRequest,
                message: format!("Bad request: {e}"),
            },
            AppError::PathRejection(e) => ErrorResponse {
                error: ErrorResponseVariant::BadRequest,
                message: format!("Bad request: {e}"),
            },
            AppError::ValidationError(errors) => ErrorResponse {
                error: ErrorResponseVariant::BadRequest,
                message: format!("Validation error: {errors:?}"),
            },
            AppError::JsonRejection(e) => ErrorResponse {
                error: ErrorResponseVariant::BadRequest,
                message: format!("Bad request: {e}"),
            },
            AppError::InternalServerError
            | AppError::MissingEnvVar(_)
            | AppError::ConfigLoadError(_)
            | AppError::DatabaseError(_)
            | AppError::TemplateError(_)
            | AppError::ServerError(_) => ErrorResponse {
                error: ErrorResponseVariant::InternalServerError,
                message: "An internal server error occurred.".to_string(),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{form::ValidationError, test_utils::response_body_string};
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
        let app = Router::new()
            .route(
                "/",
                get(|| async { AppError::NotFound("missing".to_string()) }),
            )
            .layer(middleware::from_fn(render_error_pages));

        let response = app
            .oneshot(Request::builder().uri("/").body(Body::empty()).unwrap())
            .await
            .expect("response");

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
        let body = response_body_string(response).await;
        assert!(body.contains("Error 404"));
        assert!(body.contains("missing"));
    }

    #[tokio::test]
    async fn validation_error_maps_to_bad_request() {
        let app = Router::new()
            .route(
                "/",
                get(|| async {
                    let errors = vec![("name".to_string(), ValidationError::InvalidValue)];
                    AppError::ValidationError(errors)
                }),
            )
            .layer(middleware::from_fn(render_error_pages));
        let response = app
            .oneshot(Request::builder().uri("/").body(Body::empty()).unwrap())
            .await
            .expect("response");

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
        let body = response_body_string(response).await;
        assert!(body.contains("Validation error"));
    }

    #[tokio::test]
    async fn database_error_maps_to_internal_server_error() {
        let app = Router::new()
            .route(
                "/",
                get(|| async { AppError::DatabaseError(sqlx::Error::RowNotFound) }),
            )
            .layer(middleware::from_fn(render_error_pages));
        let response = app
            .oneshot(Request::builder().uri("/").body(Body::empty()).unwrap())
            .await
            .expect("response");

        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
    }
}
