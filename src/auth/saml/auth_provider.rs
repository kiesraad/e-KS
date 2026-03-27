use axum::{
    Form, Router,
    extract::State,
    response::IntoResponse,
    routing::{get, post},
};
use openssl::{
    pkey::{PKey, Private},
    rsa::Rsa,
    x509::X509,
};
use samael::{
    metadata::EntityDescriptor,
    service_provider::{ServiceProvider, ServiceProviderBuilder},
    signature::{DigestAlgorithm, Signature, SignatureAlgorithm},
    traits::ToXml,
};
use serde::Deserialize;

use crate::{AppError, AppState, auth::saml::ActiveAuthnRequests};

pub struct AuthProvider {
    sp: ServiceProvider,
    private_key: PKey<Private>,
    public_key: X509,
    active_authn_requests: ActiveAuthnRequests,
}

impl AuthProvider {
    pub async fn new(idp_metadata_url: String) -> Result<Self, AppError> {
        let resp = reqwest::get(idp_metadata_url).await?.text().await?;
        let idp_metadata: EntityDescriptor = samael::metadata::de::from_str(&resp)
            .map_err(|_| AppError::InternalServerError)
            .unwrap();

        let public_key = X509::from_pem(
            &std::fs::read("./development/publickey.cer")
                .map_err(|_| AppError::InternalServerError)?,
        )
        .map_err(|_| AppError::InternalServerError)?;
        let private_key = Rsa::private_key_from_pem(
            &std::fs::read("./development/privatekey.pem")
                .map_err(|_| AppError::InternalServerError)?,
        )
        .map_err(|_| AppError::InternalServerError)?;
        let private_key = PKey::from_rsa(private_key).unwrap();

        let sp = ServiceProviderBuilder::default()
            .entity_id("e-ks".to_string())
            .key(private_key.clone())
            .certificate(public_key.clone())
            .allow_idp_initiated(false)
            .idp_metadata(idp_metadata)
            .acs_url("http://localhost:3000/saml/acs".to_string())
            .slo_url("http://localhost:3000/saml/logout".to_string())
            .build()
            .map_err(|_| AppError::InternalServerError)?;

        Ok(AuthProvider {
            sp,
            private_key,
            public_key,
            active_authn_requests: ActiveAuthnRequests::default(),
        })
    }
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/saml/login", get(saml_login))
        .route("/saml/metadata", get(saml_metadata))
        .route("/saml/acs", post(saml_acs))
}

async fn saml_login(State(state): State<AppState>) -> impl IntoResponse {
    let authn_request = state
        .auth_provider
        .sp
        .make_authentication_request(
            &state
                .auth_provider
                .sp
                .sso_binding_location(samael::metadata::HTTP_REDIRECT_BINDING)
                .unwrap(),
        )
        .unwrap();

    state
        .auth_provider
        .active_authn_requests
        .add(authn_request.id.clone());

    let login_url = authn_request
        .signed_redirect("", state.auth_provider.private_key.clone())
        .unwrap()
        .unwrap();

    axum::response::Redirect::temporary(login_url.as_str())
}

async fn saml_metadata(State(state): State<AppState>) -> Result<impl IntoResponse, AppError> {
    let mut metadata = state.auth_provider.sp.metadata().unwrap();

    let root_id = format!("_{}", uuid::Uuid::new_v4().simple());
    metadata.id = Some(root_id.clone());

    let mut sig = Signature::template(&root_id, &state.auth_provider.public_key.to_der().unwrap());
    sig.signed_info.signature_method.algorithm = SignatureAlgorithm::RsaSha256;
    sig.signed_info.reference[0].digest_method.algorithm = DigestAlgorithm::Sha256;
    metadata.signature = Some(sig);

    let unsigned_xml = TryInto::<crate::auth::saml::structs::EntityDescriptor>::try_into(metadata)?
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
    let active_authn_requests = state.auth_provider.active_authn_requests.list_all();
    let possible_request_ids = active_authn_requests
        .iter()
        .map(String::as_str)
        .collect::<Vec<_>>();

    let t = state
        .auth_provider
        .sp
        .parse_base64_response(&saml_response, Some(&possible_request_ids))
        .unwrap();

    if let Some(in_response_to) = t
        .subject
        .as_ref()
        .and_then(|subject| subject.subject_confirmations.as_ref())
        .and_then(|confirmations| {
            confirmations.iter().find_map(|confirmation| {
                confirmation
                    .subject_confirmation_data
                    .as_ref()
                    .and_then(|data| data.in_response_to.as_deref())
            })
        })
    {
        state
            .auth_provider
            .active_authn_requests
            .remove(in_response_to);
    }

    format!("Received: {:#?}", t)
}

#[cfg(test)]
mod tests {
    use axum::{
        body::Body,
        http::{Request, StatusCode},
    };
    use bytes::{Buf, BytesMut};
    use tower::ServiceExt;

    use super::*;

    #[tokio::test]
    async fn signed_sp_metadata_verifies() {
        let app_state = AppState::new_for_tests().await;
        let router = router().with_state(app_state.clone());

        let response = router
            .oneshot(Request::get("/saml/metadata").body(Body::empty()).unwrap())
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let bytes = axum::body::to_bytes(response.into_body(), 64000)
            .await
            .unwrap();

        // retrieved SP metadata should have a valid signature
        let metadata: EntityDescriptor =
            samael::metadata::de::from_reader(bytes.clone().reader()).unwrap();
        assert_eq!(metadata.entity_id, Some("e-ks".to_owned()));
        assert!(
            samael::crypto::verify_signed_xml(
                bytes.clone(),
                &app_state.auth_provider.public_key.to_der().unwrap(),
                Some("ID"),
            )
            .is_ok()
        );

        // let's mess up the signature
        let sig_start = b"<ds:SignatureValue>";
        let position = bytes
            .windows(sig_start.len())
            .position(|window| window == sig_start)
            .unwrap()
            + sig_start.len();
        let mut bytes_wrong_sig = BytesMut::from(bytes);
        for i in position..position + 10 {
            bytes_wrong_sig[i] = b'X';
        }

        // now it no longer verifies
        assert!(matches!(
            samael::crypto::verify_signed_xml(
                bytes_wrong_sig,
                &app_state.auth_provider.public_key.to_der().unwrap(),
                Some("ID"),
            )
            .unwrap_err(),
            samael::crypto::Error::InvalidSignature
        ));
    }
}
