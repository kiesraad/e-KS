//! SAML message builders: AuthnRequest, ArtifactResolve, LogoutRequest.

use secrecy::ExposeSecret;

use crate::{
    error::Result,
    keys::KeyPair,
    saml::{
        crypto::sign,
        loa::MINIMUM_LOA,
        xml_builder::{
            ArtifactResolveArgs, AuthnRequestArgs, LogoutRequestArgs, build_artifact_resolve,
            build_authn_request, build_logout_request, generate_id, now_utc,
        },
    },
};

pub struct CreatedMessage {
    pub id: String,
    pub xml: String,
}

pub fn create_authn_request(
    entity_id: &str,
    service_uuid: &str,
    sso_url: &str,
    signing_key: &KeyPair,
    preselected_ad_entity_id: Option<&str>,
    acs_url: Option<&str>,
) -> Result<CreatedMessage> {
    let id = generate_id();
    let issue_instant = now_utc();

    let xml = build_authn_request(AuthnRequestArgs {
        id: &id,
        issue_instant: &issue_instant,
        destination: sso_url,
        issuer: entity_id,
        acs_url,
        intended_audience: entity_id,
        service_uuid,
        // eID §7.6.3.2 / TVS T6: request the DV's minimum LoA so the RD never
        // authenticates below it; the response is still re-checked on arrival.
        requested_loa_uri: Some(MINIMUM_LOA.as_request_uri()),
        preselected_ad_entity_id,
        signing_cert_base64: &signing_key.cert_base64,
    })?;

    let signed = sign(&xml, signing_key.key_pem.expose_secret())?;

    Ok(CreatedMessage { id, xml: signed })
}

pub fn create_artifact_resolve(
    artifact: &str,
    entity_id: &str,
    ars_url: &str,
    signing_key: &KeyPair,
) -> Result<CreatedMessage> {
    let id = generate_id();
    let issue_instant = now_utc();

    let xml = build_artifact_resolve(ArtifactResolveArgs {
        id: &id,
        issue_instant: &issue_instant,
        destination: ars_url,
        issuer: entity_id,
        artifact,
        signing_cert_base64: &signing_key.cert_base64,
    })?;

    let signed = sign(&xml, signing_key.key_pem.expose_secret())?;

    Ok(CreatedMessage { id, xml: signed })
}

pub fn create_logout_request(
    name_id: &str,
    entity_id: &str,
    slo_url: &str,
    signing_key: &KeyPair,
) -> Result<CreatedMessage> {
    let id = generate_id();
    let issue_instant = now_utc();

    let xml = build_logout_request(LogoutRequestArgs {
        id: &id,
        issue_instant: &issue_instant,
        destination: slo_url,
        issuer: entity_id,
        name_id,
        signing_cert_base64: &signing_key.cert_base64,
    })?;

    let signed = sign(&xml, signing_key.key_pem.expose_secret())?;

    Ok(CreatedMessage { id, xml: signed })
}
