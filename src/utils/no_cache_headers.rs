use axum::http::{HeaderMap, HeaderValue};
use reqwest::header;

use crate::AppError;

pub fn generate_attachment_headers(
    filename: &str,
    content_type: HeaderValue,
) -> Result<HeaderMap, AppError> {
    let mut headers = HeaderMap::new();
    headers.insert(header::CONTENT_TYPE, content_type);
    headers.insert(
        header::CONTENT_DISPOSITION,
        HeaderValue::from_str(&format!(r#"attachment; filename="{}""#, filename)).map_err(
            |_| {
                tracing::error!("invalid filename for content disposition: {}", filename);

                AppError::InternalServerError
            },
        )?,
    );
    headers.insert(
        header::CACHE_CONTROL,
        HeaderValue::from_static("no-store, no-cache, must-revalidate, max-age=0"),
    );
    headers.insert(header::PRAGMA, HeaderValue::from_static("no-cache"));
    headers.insert(header::EXPIRES, HeaderValue::from_static("0"));
    Ok(headers)
}
