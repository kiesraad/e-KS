//! Renders PDFs from Typst templates, either in-process (with the
//! `embed-typst` feature enabled) or via an external typst-webservice HTTP
//! server.

use crate::AppError;
use axum::body::Body;
use serde::Serialize;
use tracing::debug;

#[cfg(feature = "embed-typst")]
use std::sync::Arc;
#[cfg(feature = "embed-typst")]
use typst_webservice::PdfContext;

/// A renderer that turns Typst templates into PDFs.
#[derive(Clone)]
pub enum TypstRenderer {
    /// Render PDFs by calling an external typst-webservice over HTTP.
    Http(String),
    /// Render PDFs in-process using the embedded typst-webservice library.
    #[cfg(feature = "embed-typst")]
    Embedded(Arc<PdfContext>),
}

/// Request describing one PDF in a batch render call.
pub struct BatchRenderRequest {
    pub template: &'static str,
    pub file_name: String,
    pub input: serde_json::Value,
}

#[derive(Serialize)]
struct HttpBatchRenderRequest {
    template: &'static str,
    file_name: String,
    input: serde_json::Value,
}

impl TypstRenderer {
    pub fn http(base_url: String) -> Self {
        Self::Http(base_url)
    }

    #[cfg(feature = "embed-typst")]
    pub fn embedded(context: Arc<PdfContext>) -> Self {
        Self::Embedded(context)
    }

    /// Render a single PDF and return its bytes as an axum [`Body`].
    pub async fn render_pdf<T: Serialize>(
        &self,
        template: &'static str,
        file_name: &str,
        input: &T,
    ) -> Result<Body, AppError> {
        match self {
            Self::Http(base_url) => {
                let url = format!("{base_url}/render-pdf/{template}/{file_name}");
                debug!("Sending PDF generation request to {url}");
                let stream = reqwest::Client::new()
                    .get(url)
                    .json(input)
                    .send()
                    .await?
                    .error_for_status()?
                    .bytes_stream();
                Ok(Body::from_stream(stream))
            }
            #[cfg(feature = "embed-typst")]
            Self::Embedded(context) => {
                debug!("Rendering {template} in-process using embedded Typst");
                let context = context.clone();
                let template = template.to_string();
                let input = serde_json::to_value(input)?;
                let bytes = tokio::task::spawn_blocking(move || {
                    PdfContext::render(context, template, input)
                })
                .await
                .map_err(typst_webservice::AppError::from)??;
                Ok(Body::from(bytes))
            }
        }
    }

    /// Render a batch of PDFs into a ZIP archive and return the bytes as an
    /// axum [`Body`].
    pub async fn render_batch(&self, requests: Vec<BatchRenderRequest>) -> Result<Body, AppError> {
        match self {
            Self::Http(base_url) => {
                let url = format!("{base_url}/render-pdf/batch");
                debug!("Sending PDF ZIP generation request to {url}");
                let payload: Vec<HttpBatchRenderRequest> = requests
                    .into_iter()
                    .map(|req| HttpBatchRenderRequest {
                        template: req.template,
                        file_name: req.file_name,
                        input: req.input,
                    })
                    .collect();

                let stream = reqwest::Client::new()
                    .post(url)
                    .json(&payload)
                    .send()
                    .await?
                    .error_for_status()?
                    .bytes_stream();
                Ok(Body::from_stream(stream))
            }
            #[cfg(feature = "embed-typst")]
            Self::Embedded(context) => {
                debug!(
                    "Rendering {} PDFs in-process using embedded Typst",
                    requests.len()
                );
                let context = context.clone();
                let requests = requests
                    .into_iter()
                    .map(|req| typst_webservice::BatchRenderRequest {
                        template: req.template.to_string(),
                        file_name: req.file_name,
                        input: req.input,
                    })
                    .collect();
                let bytes = PdfContext::render_batch(context, requests).await?;
                Ok(Body::from(bytes))
            }
        }
    }
}
