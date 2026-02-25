use crate::{AppError, submit::H1};
use bytes::Bytes;
use futures_core::Stream;
use reqwest::Method;
use tracing::{debug};

pub enum Pdf {
    H1(H1),
}

impl Pdf {
    pub async fn generate(self) -> Result<impl Stream<Item = reqwest::Result<Bytes>>, AppError> {
        debug!("Sending PDF generation request");
        Ok(match self {
            Pdf::H1(h1) => reqwest::Client::new()
                .request(Method::GET, "http://localhost:8080/render-pdf/model-h-1.typ/h1.pdf")
                .json(&h1),
        }
        .send()
        .await?
        .bytes_stream())
    }
}
