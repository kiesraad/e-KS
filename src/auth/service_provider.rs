use axum::{
    Form, Router,
    extract::State,
    response::IntoResponse,
    routing::{get, post},
};
use openssl::pkey::{PKey, Private};
use samael::{
    metadata::EntityDescriptor,
    service_provider::{ServiceProvider, ServiceProviderBuilder},
    signature::{DigestAlgorithm, Signature, SignatureAlgorithm},
    traits::ToXml,
};
use serde::Deserialize;

use crate::{AppError, AppState};

pub struct AuthProvider {
    sp: ServiceProvider,
    private_key: PKey<Private>,
    public_key: openssl::x509::X509,
}

impl AuthProvider {
    pub async fn new(idp_metadata_url: String) -> Result<Self, AppError> {
        let resp = reqwest::get(idp_metadata_url).await?.text().await?;
        let idp_metadata: EntityDescriptor = samael::metadata::de::from_str(&resp)
            .map_err(|_| AppError::InternalServerError)
            .unwrap();

        let public_key = openssl::x509::X509::from_pem(
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
            .certificate(public_key.clone())
            .allow_idp_initiated(true) // TODO: disable this and keep track of requests IDs
            .idp_metadata(idp_metadata)
            .acs_url("http://localhost:8080/saml/acs".to_string())
            .slo_url("http://localhost:8080/saml/slo".to_string())
            .build()
            .map_err(|_| AppError::InternalServerError)?;

        Ok(AuthProvider {
            sp,
            private_key,
            public_key,
        })
    }
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/saml/metadata", get(saml_metadata))
        .route("/saml/acs", post(saml_acs))
}

fn random_xml_id(prefix: &str) -> String {
    format!("_{}{}", prefix, uuid::Uuid::new_v4().simple())
}

async fn saml_metadata(State(state): State<AppState>) -> Result<impl IntoResponse, AppError> {
    let mut metadata = state.auth_provider.sp.metadata().unwrap();

    let root_id = random_xml_id("md");
    metadata.id = Some(root_id.clone());

    let mut sig = Signature::template(&root_id, &state.auth_provider.public_key.to_der().unwrap());
    sig.signed_info.signature_method.algorithm = SignatureAlgorithm::RsaSha256;
    sig.signed_info.reference[0].digest_method.algorithm = DigestAlgorithm::Sha256;
    metadata.signature = Some(sig);

    let unsigned_xml = TryInto::<crate::auth::saml_structs::EntityDescriptor>::try_into(metadata)
        .unwrap()
        .to_string()
        .map_err(|_| AppError::InternalServerError)?;

    let signed_metadata = samael::crypto::sign_xml(
        unsigned_xml.as_bytes(),
        &state
            .auth_provider
            .private_key
            .private_key_to_der()
            .unwrap(),
    )
    .unwrap();

    Ok((
        [(reqwest::header::CONTENT_TYPE, "application/xml")],
        signed_metadata,
    ))
}

#[derive(Deserialize)]
struct SAMLResponse {
    #[serde(rename = "SAMLResponse")]
    saml_response: String,
}

async fn saml_acs(
    State(state): State<AppState>,
    Form(SAMLResponse { saml_response }): Form<SAMLResponse>,
) -> impl IntoResponse {
    let t = state
        .auth_provider
        .sp
        .parse_base64_response(&saml_response, Some(&["a_possible_request_id"])) // TODO: use a list of active request IDs
        .unwrap();
    format!("Received: {:#?}", t)
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
