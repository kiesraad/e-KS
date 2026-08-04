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

/// Inputs to [`create_authn_request`]: the DV identity, the RD SSO endpoint the
/// request is destined for, and the signing key.
pub struct AuthnRequestSpec<'a> {
    pub entity_id: &'a str,
    pub service_uuid: &'a str,
    pub sso_url: &'a str,
    pub signing_key: &'a KeyPair,
    /// AD to pre-select via `Scoping/IDPList` (eID §7.3); `None` sends no Scoping.
    pub preselected_ad_entity_id: Option<&'a str>,
    /// Explicit ACS URL, Test-environment only (see `AuthnRequestArgs::acs_url`).
    pub acs_url: Option<&'a str>,
}

pub fn create_authn_request(spec: &AuthnRequestSpec<'_>) -> Result<CreatedMessage> {
    let id = generate_id();
    let issue_instant = now_utc();

    let xml = build_authn_request(AuthnRequestArgs {
        id: &id,
        issue_instant: &issue_instant,
        destination: spec.sso_url,
        issuer: spec.entity_id,
        acs_url: spec.acs_url,
        intended_audience: spec.entity_id,
        service_uuid: spec.service_uuid,
        // eID §7.6.3.2 / TVS T6: request the DV's minimum LoA so the RD never
        // authenticates below it; the response is still re-checked on arrival.
        requested_loa_uri: Some(MINIMUM_LOA.as_request_uri()),
        preselected_ad_entity_id: spec.preselected_ad_entity_id,
        signing_cert_base64: &spec.signing_key.cert_base64,
    })?;

    let signed = sign(&xml, spec.signing_key.key_pem.expose_secret())?;

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
