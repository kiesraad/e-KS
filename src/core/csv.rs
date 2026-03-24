use axum::{
    body::Body,
    http::{HeaderValue, Response},
    response::IntoResponse,
};
use csv::Writer;
use serde::Serialize;

use crate::{AppError, utils::no_cache_headers};

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
        let headers = no_cache_headers::generate_attachment_headers(
            self.filename.as_str(),
            HeaderValue::from_static("text/csv"),
        )?;

        Ok((headers, data).into_response())
    }
}
