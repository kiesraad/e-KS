use axum::{Router, extract::State, response::IntoResponse, routing::get};
use openssl::pkey::{PKey, Private};
use samael::{
    metadata::{ContactPerson, ContactType, EntityDescriptor},
    service_provider::{ServiceProvider, ServiceProviderBuilder},
    traits::ToXml,
};

use crate::{AppError, AppState};

pub struct AuthProvider {
    sp: ServiceProvider,
    private_key: PKey<Private>,
}

impl AuthProvider {
    pub async fn new(idp_metadata_url: String) -> Result<Self, AppError> {
        let resp = reqwest::get(idp_metadata_url).await?.text().await?;
        let idp_metadata: EntityDescriptor = samael::metadata::de::from_str(&resp)
            .map_err(|_| AppError::InternalServerError)
            .unwrap();

        let pub_key = openssl::x509::X509::from_pem(
            &std::fs::read("./publickey.cer").map_err(|_| AppError::InternalServerError)?,
        )
        .map_err(|_| AppError::InternalServerError)?;
        let private_key = openssl::rsa::Rsa::private_key_from_pem(
            &std::fs::read("./privatekey.pem").map_err(|_| AppError::InternalServerError)?,
        )
        .map_err(|_| AppError::InternalServerError)?;
        let private_key = PKey::from_rsa(private_key).unwrap();

        let sp = ServiceProviderBuilder::default()
            .entity_id("test-sp".to_string())
            .key(private_key.clone())
            .certificate(pub_key)
            .allow_idp_initiated(true) // TODO: disable this and keep track of requests IDs
            .contact_person(ContactPerson {
                sur_name: Some("Bob".to_string()),
                contact_type: Some(ContactType::Technical.value().to_string()),
                ..ContactPerson::default()
            })
            .idp_metadata(idp_metadata)
            .acs_url("http://localhost:8080/saml/acs".to_string())
            .slo_url("http://localhost:8080/saml/slo".to_string())
            .build()
            .map_err(|_| AppError::InternalServerError)?;

        Ok(AuthProvider { sp, private_key })
    }
}

pub fn router() -> Router<AppState> {
    Router::new().route("/saml/metadata", get(saml_metadata))
}

async fn saml_metadata(State(state): State<AppState>) -> Result<impl IntoResponse, AppError> {
    let metadata_xml = state
        .auth_provider
        .sp
        .metadata()
        .map_err(|_| AppError::InternalServerError)?
        .to_string()
        .map_err(|_| AppError::InternalServerError)?;
    Ok((
        [(reqwest::header::CONTENT_TYPE, "application/xml")],
        metadata_xml,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_auth_provider() {
        let auth_provider = AuthProvider::new(
            "http://localhost:9001/simplesaml/saml2/idp/metadata.php".to_string(),
        )
        .await
        .unwrap();
        assert_eq!(auth_provider.sp.entity_id, Some("test-sp".to_string()));
    }
}
