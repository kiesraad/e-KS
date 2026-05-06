use crate::{
    AppError,
    core::typst_renderer::{BatchRenderRequest, TypstRenderer},
    utils::no_cache_headers,
};
use axum::{
    body::{Body, to_bytes},
    http::{HeaderValue, Response},
    response::IntoResponse,
};
use serde::Serialize;

const PDF_CONTENT_TYPE: &str = "application/pdf";
const ZIP_CONTENT_TYPE: &str = "application/zip";

pub trait Pdf: Sized + Serialize {
    fn typst_template_name(&self) -> &'static str;

    fn filename(&self) -> &str;

    async fn generate(self, renderer: &TypstRenderer) -> Result<Response<Body>, AppError> {
        let template = self.typst_template_name();
        let filename = self.filename().to_owned();

        let body = renderer.render_pdf(template, &filename, &self).await?;
        let bytes = to_bytes(body, usize::MAX)
            .await
            .map_err(|_| AppError::InternalServerError)?;

        tracing::info!(
            file_name = %filename,
            content_type = PDF_CONTENT_TYPE,
            size_bytes = bytes.len(),
            "file download served",
        );

        let headers = no_cache_headers::generate_attachment_headers(
            &filename,
            HeaderValue::from_static(PDF_CONTENT_TYPE),
        )?;

        Ok((headers, bytes).into_response())
    }
}

pub struct PdfZip<T>
where
    T: Pdf,
{
    pub filename: String,
    pub pdfs: Vec<T>,
}

impl<T> PdfZip<T>
where
    T: Pdf,
{
    pub async fn generate(self, renderer: &TypstRenderer) -> Result<Response<Body>, AppError> {
        let mut requests = Vec::with_capacity(self.pdfs.len());
        for pdf in self.pdfs {
            requests.push(BatchRenderRequest {
                template: pdf.typst_template_name(),
                file_name: pdf.filename().to_owned(),
                input: serde_json::to_value(pdf)?,
            });
        }

        let body = renderer.render_batch(requests).await?;
        let bytes = to_bytes(body, usize::MAX)
            .await
            .map_err(|_| AppError::InternalServerError)?;

        tracing::info!(
            file_name = %self.filename,
            content_type = ZIP_CONTENT_TYPE,
            size_bytes = bytes.len(),
            "file download served",
        );

        let headers = no_cache_headers::generate_attachment_headers(
            self.filename.as_str(),
            HeaderValue::from_static(ZIP_CONTENT_TYPE),
        )?;

        Ok((headers, bytes).into_response())
    }
}
