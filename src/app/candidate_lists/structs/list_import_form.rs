//! unused and untested at this moment
//! might be useful as example when we decide we do need multipart support

use axum::extract::Multipart;
use serde::{Deserialize, Serialize};
use validate::Validate;

use crate::{AppError, FromMultipart, TokenValue};

/// Trait for populating a struct from multipart form fields.
pub trait FromMultipart: Sized {
    fn from_multipart(
        multipart: &mut Multipart,
    ) -> impl std::future::Future<Output = Result<Self, AppError>> + Send;
}

/// Wrapper that extracts multipart form data into a struct.
pub struct MultipartForm<T>(pub T);

#[derive(Default, Debug, Clone)]
pub struct ListImport {
    pub file_data: Vec<u8>,
}

#[derive(Default, Serialize, Deserialize, Clone, Debug, Validate)]
#[validate(target = "ListImport")]
#[serde(default)]
pub struct ListImportForm {
    pub file_data: Vec<u8>,
    #[validate(csrf)]
    pub csrf_token: TokenValue,
}

impl FromMultipart for ListImportForm {
    async fn from_multipart(multipart: &mut Multipart) -> Result<Self, AppError> {
        let mut form = ListImportForm::default();

        while let Some(field) = multipart.next_field().await? {
            match field.name() {
                Some("csrf_token") => {
                    form.csrf_token = TokenValue(field.text().await?);
                }
                Some("import_file") => {
                    form.file_data = field.bytes().await?.to_vec();
                }
                _ => { // ignore unknown form field
                }
            }
        }
        Ok(form)
    }
}
