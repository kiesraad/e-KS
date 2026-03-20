use std::fmt::Display;

use axum::{
    body::Body,
    http::{HeaderValue, Response},
    response::IntoResponse,
};
use csv::{Reader, Writer};
use serde::{Serialize, de::DeserializeOwned};

use crate::{AppError, utils::no_cache_headers};

pub enum CsvError {
    FormatError {
        candidate_number: usize,
        message: String,
    },
    ParseError {
        candidate_number: usize,
        field_name: String,
        field_value: String,
        message: String,
    },
}

impl Display for CsvError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CsvError::FormatError {
                candidate_number,
                message,
            } => write!(f, "Error with candidate #{candidate_number}: {message}"),
            CsvError::ParseError {
                candidate_number,
                field_name,
                field_value,
                message,
            } => write!(
                f,
                "Error with candidate #{candidate_number} on field \"{field_name}\" ({field_value}): {message}"
            ),
        }
    }
}

pub struct Csv<T: Serialize + DeserializeOwned> {
    pub records: Vec<T>,
    pub filename: String,
}

impl<T: Serialize + DeserializeOwned> Csv<T> {
    pub fn generate_csv_response(&self) -> Result<Response<Body>, AppError> {
        let mut csv_writer = Writer::from_writer(vec![]);
        for record in &self.records {
            if csv_writer.serialize(record).is_err() {
                return Err(AppError::InternalServerError);
            }
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

    pub fn from_bytes(data: &[u8]) -> Result<Vec<T>, Vec<CsvError>> {
        let mut records = vec![];
        let mut errors = vec![];

        Reader::from_reader(data)
            .deserialize::<T>()
            .enumerate()
            .for_each(|(count, res)| match res {
                Ok(record) => records.push(record),
                Err(error) => errors.push(CsvError::FormatError {
                    candidate_number: count + 1,
                    message: error.to_string(),
                }),
            });
        if errors.is_empty() {
            Ok(records)
        } else {
            Err(errors)
        }
    }
}
