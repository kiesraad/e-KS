use crate::AppError;
use axum::{
    body::Body,
    http::{HeaderMap, HeaderValue, Response, header},
    response::IntoResponse,
};
use reqwest::Method;
use serde::Serialize;
use tracing::debug;

pub trait Pdf: Sized + Serialize {
    fn typst_template_name(&self) -> &'static str;

    fn filename(&self) -> &str;

    async fn generate(self, typst_url: &str) -> Result<Response<Body>, AppError> {
        let url = format!(
            "{typst_url}/render-pdf/{}/{}",
            self.typst_template_name(),
            self.filename()
        );

        debug!("Sending PDF generation request to {url}");

        let typst_response = reqwest::Client::new()
            .request(Method::GET, url)
            .json(&self)
            .send()
            .await?
            .error_for_status()?
            .bytes_stream();

        let mut headers = HeaderMap::new();
        headers.insert(
            header::CONTENT_TYPE,
            HeaderValue::from_static("application/pdf"),
        );
        headers.insert(
            header::CONTENT_DISPOSITION,
            HeaderValue::from_str(&format!(r#"attachment; filename="{}""#, self.filename()))
                .map_err(|_| {
                    tracing::error!(
                        "invalid filename for content disposition: {}",
                        self.filename()
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

        Ok((headers, Body::from_stream(typst_response)).into_response())
    }
}

pub struct PdfZip<T>
where
    T: Pdf,
{
    pub filename: String,
    pub pdfs: Vec<T>,
}

#[derive(serde::Serialize)]
struct BatchRenderRequest {
    /// Name of the Typst template to render.
    template: &'static str,
    /// File name (including extension) for the PDF inside the archive.
    file_name: String,
    /// JSON payload injected into the Typst template.
    input: serde_json::Value,
}

impl<T> PdfZip<T>
where
    T: Pdf,
{
    pub async fn generate(self, typst_url: &str) -> Result<Response<Body>, AppError> {
        let url = format!("{typst_url}/render-pdf/batch");

        debug!("Sending PDF ZIP generation request to {url}");

        let mut payload = vec![];
        for pdf in self.pdfs {
            payload.push(BatchRenderRequest {
                template: pdf.typst_template_name(),
                file_name: pdf.filename().to_owned(),
                input: serde_json::to_value(pdf)?,
            });
        }

        let typst_response = reqwest::Client::new()
            .request(Method::POST, url)
            .json(&payload)
            .send()
            .await?
            .error_for_status()?
            .bytes_stream();

        let mut headers = HeaderMap::new();
        headers.insert(
            header::CONTENT_TYPE,
            HeaderValue::from_static("application/zip"),
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

        Ok((headers, Body::from_stream(typst_response)).into_response())
    }
}
