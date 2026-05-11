//! Renders PDFs from Typst templates, either in-process (with the
//! `embed-typst` feature enabled) or via an external typst-webservice HTTP
//! server.

use crate::AppError;
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

impl TypstRenderer {
    pub fn http(base_url: String) -> Self {
        Self::Http(base_url)
    }

    #[cfg(feature = "embed-typst")]
    pub fn embedded(context: Arc<PdfContext>) -> Self {
        Self::Embedded(context)
    }

    /// Render a single PDF and return its bytes in memory.
    pub async fn render_pdf<T: Serialize>(
        &self,
        template: &'static str,
        file_name: &str,
        input: &T,
    ) -> Result<Vec<u8>, AppError> {
        match self {
            Self::Http(base_url) => {
                let url = format!("{base_url}/render-pdf/{template}/{file_name}");
                debug!("Sending PDF generation request to {url}");
                let bytes = reqwest::Client::new()
                    .get(url)
                    .json(input)
                    .send()
                    .await?
                    .error_for_status()?
                    .bytes()
                    .await?;
                Ok(bytes.to_vec())
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
                Ok(bytes)
            }
        }
    }
}
