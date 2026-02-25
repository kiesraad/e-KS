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
            HeaderValue::from_str(&format!(
                r#"attachment; filename="{}""#,
                self.filename()
            ))
            .expect("Must be valid header value"),
        );

        Ok((headers, Body::from_stream(typst_response)).into_response())
    }
}
