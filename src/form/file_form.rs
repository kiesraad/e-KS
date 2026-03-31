use axum::{
    body::Bytes,
    extract::{FromRequest, Multipart, Request},
};

use crate::{AppError, TokenValue};

#[derive(Debug, Default)]
pub struct FileForm {
    pub csrf_token: TokenValue,
    pub file_data: Option<Bytes>,
}

impl<S> FromRequest<S> for FileForm
where
    S: Sync + Send,
{
    type Rejection = AppError;

    async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
        let mut multipart = Multipart::from_request(req, state).await?;
        let mut form = FileForm::default();

        while let Some(field) = multipart.next_field().await? {
            match field.name() {
                Some("csrf_token") => {
                    form.csrf_token = TokenValue(field.text().await?);
                }
                Some("file_data") => {
                    form.file_data = Some(field.bytes().await?);
                }
                _ => {}
            }
        }

        Ok(form)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{body::Body, extract::FromRequest, http::Request};

    fn multipart_request(path: &str, csrf_token: &str, csv: &str, boundary: &str) -> Request<Body> {
        Request::builder()
            .uri(path)
            .header(
                "Content-Type",
                format!("multipart/form-data; boundary={boundary}"),
            )
            .body(Body::from(format!(
                "--{boundary}\r\nContent-Disposition: form-data; name=\"csrf_token\"\r\n\r\n{csrf_token}\r\n--{boundary}\r\nContent-Disposition: form-data; name=\"file_data\"; filename=\"candidates.csv\"\r\nContent-Type: text/csv\r\n\r\n{csv}\r\n--{boundary}--\r\n"
            )))
            .unwrap()
    }

    #[tokio::test]
    async fn extracts_uploaded_file_and_csrf_token() -> Result<(), AppError> {
        let request = multipart_request(
            "/candidate-lists/import",
            "csrf-token",
            "voorletters,achternaam\r\nJ.,Berg",
            "----eks-boundary",
        );

        let form = FileForm::from_request(request, &()).await?;

        assert_eq!(form.csrf_token.0, "csrf-token");
        assert_eq!(
            form.file_data.as_deref(),
            Some(&b"voorletters,achternaam\r\nJ.,Berg"[..])
        );

        Ok(())
    }
}
