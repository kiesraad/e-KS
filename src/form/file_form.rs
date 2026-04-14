use axum::{
    body::Bytes,
    extract::{FromRequest, Multipart, Request},
};

use crate::{AppError, TokenValue};

const MAX_FILE_NAME_LEN: usize = 255;

#[derive(Debug, Default)]
pub struct FileForm {
    pub csrf_token: TokenValue,
    pub file_name: Option<String>,
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
                    form.file_name = sanitize_file_name(field.file_name());
                    form.file_data = Some(field.bytes().await?);
                }
                _ => {}
            }
        }

        Ok(form)
    }
}

/// Strip path components and control characters from an uploaded file name,
/// and truncate to a bounded length. Returns `None` if the result is empty.
fn sanitize_file_name(raw: Option<&str>) -> Option<String> {
    let basename = raw?.rsplit(['/', '\\']).next().unwrap_or_default();

    let cleaned: String = basename
        .chars()
        .filter(|c| !c.is_control())
        .take(MAX_FILE_NAME_LEN)
        .collect();

    let trimmed = cleaned.trim();
    (!trimmed.is_empty()).then(|| trimmed.to_string())
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
        assert_eq!(form.file_name.as_deref(), Some("candidates.csv"));
        assert_eq!(
            form.file_data.as_deref(),
            Some(&b"voorletters,achternaam\r\nJ.,Berg"[..])
        );

        Ok(())
    }

    #[test]
    fn sanitize_file_name_strips_paths_and_control_chars() {
        assert_eq!(
            sanitize_file_name(Some("../../etc/passwd")).as_deref(),
            Some("passwd")
        );
        assert_eq!(
            sanitize_file_name(Some(r"C:\Users\x\report.csv")).as_deref(),
            Some("report.csv")
        );
        assert_eq!(
            sanitize_file_name(Some("bad\x00name\n.csv")).as_deref(),
            Some("badname.csv")
        );
        assert_eq!(sanitize_file_name(Some("   ")), None);
        assert_eq!(sanitize_file_name(Some("")), None);
        assert_eq!(sanitize_file_name(None), None);
    }

    #[test]
    fn sanitize_file_name_truncates_to_max_length() {
        let long = "a".repeat(500) + ".csv";
        let sanitized = sanitize_file_name(Some(&long)).unwrap();
        assert_eq!(sanitized.chars().count(), MAX_FILE_NAME_LEN);
    }
}
