use axum::{
    body::Body,
    http::{HeaderMap, HeaderValue, Response},
    response::IntoResponse,
};
use csv::Writer;
use reqwest::header;
use serde::Serialize;

use crate::AppError;

pub struct Csv<T: Serialize> {
    pub records: Vec<T>,
    pub filename: String,
}

impl<T: Serialize> Csv<T> {
    pub fn generate_csv_response(&self) -> Result<Response<Body>, AppError> {
        let mut csv_writer = Writer::from_writer(vec![]);
        for record in &self.records {
            csv_writer.serialize(record)?;
        }
        let data = if let Ok(data) = csv_writer.into_inner() {
            data
        } else {
            return Err(AppError::InternalServerError);
        };
        let headers = self.generate_headers()?;

        Ok((headers, data).into_response())
    }

    fn generate_headers(&self) -> Result<HeaderMap, AppError> {
        let mut headers = HeaderMap::new();
        headers.insert(
            header::CONTENT_TYPE,
            HeaderValue::from_static("text/csv"),
        );
        headers.insert(
            header::CONTENT_DISPOSITION,
            HeaderValue::from_str(&format!(r#"attachment; filename="{}""#, self.filename))
                .map_err(|_| {
                    tracing::error!(
                        "invalid filename for content disposition: {}",
                        self.filename
                    );

                    AppError::InternalServerError
                })?,
        );
        headers.insert(
            header::CACHE_CONTROL,
            HeaderValue::from_static("no-store, no-cache, must-revalidate, max-age=0"),
        );
        headers.insert(header::PRAGMA, HeaderValue::from_static("no-cache"));
        headers.insert(header::EXPIRES, HeaderValue::from_static("0"));
        Ok(headers)
    }
}
