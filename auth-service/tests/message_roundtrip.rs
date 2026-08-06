//! Tests for outgoing SAML message building and signing.
//!
//! Every message built, signed, and verified against the signing key must
//! round-trip: render, sign, parse, and verify must all cohere. Assertions are
//! structural (not byte-exact).

use auth_service::saml::{
    messages::{
        AuthnRequestSpec, create_artifact_resolve, create_authn_request, create_logout_request,
    },
    verification::verify_xml_signature,
};

mod common;
use common::load_key;

#[test]
fn authn_request_builds_signs_and_verifies() {
    let key = load_key("dv-signing-1");
    let msg = create_authn_request(&AuthnRequestSpec {
        entity_id: "urn:test:dv",
        service_uuid: "f847dc11-ac24-47b2-84a8-a057440ce56d",
        sso_url: "https://rd.example.com/sso",
        signing_key: &key,
        preselected_ad_entity_id: None,
        acs_url: None,
    })
    .expect("AuthnRequest built");

    // Structural assertions.
    assert!(msg.xml.contains("AuthnRequest"));
    assert!(msg.xml.contains("Version=\"2.0\""));
    assert!(msg.xml.contains("ForceAuthn=\"true\""));
    assert!(msg.xml.contains("<saml:Issuer>urn:test:dv</saml:Issuer>"));
    assert!(msg.xml.contains(&format!("ID=\"{}\"", msg.id)));
    assert!(
        !msg.xml.contains("Scoping"),
        "no Scoping without a preselect"
    );

    // The enveloping signature must verify against the signing key.
    let result = verify_xml_signature(&msg.xml, std::slice::from_ref(&key), None);
    assert!(
        result.is_valid(),
        "AuthnRequest signature: {:?}",
        result.errors
    );
}

#[test]
fn authn_request_with_preselect_emits_scoping_and_verifies() {
    let key = load_key("dv-signing-1");
    let ad = "urn:nl-eid-gdi:1.0:AD:00000004166909913000:entities:9002";
    let msg = create_authn_request(&AuthnRequestSpec {
        entity_id: "urn:test:dv",
        service_uuid: "uuid-1",
        sso_url: "https://rd.example.com/sso",
        signing_key: &key,
        preselected_ad_entity_id: Some(ad),
        acs_url: None,
    })
    .expect("AuthnRequest built");

    assert!(msg.xml.contains("<samlp:Scoping>"));
    assert!(msg.xml.contains(&format!("ProviderID=\"{ad}\"")));
    let result = verify_xml_signature(&msg.xml, std::slice::from_ref(&key), None);
    assert!(
        result.is_valid(),
        "AuthnRequest signature: {:?}",
        result.errors
    );
}

#[test]
fn artifact_resolve_builds_signs_and_verifies() {
    let key = load_key("dv-signing-1");
    let msg = create_artifact_resolve(
        "AAQAAGotsbEd41l9KWDK",
        "urn:test:dv",
        "https://rd.example.com/ars",
        &key,
    )
    .expect("ArtifactResolve built");

    assert!(msg.xml.contains("ArtifactResolve"));
    assert!(
        msg.xml
            .contains("<samlp:Artifact>AAQAAGotsbEd41l9KWDK</samlp:Artifact>")
    );
    assert!(msg.xml.contains("<saml:Issuer>urn:test:dv</saml:Issuer>"));
    let result = verify_xml_signature(&msg.xml, std::slice::from_ref(&key), None);
    assert!(
        result.is_valid(),
        "ArtifactResolve signature: {:?}",
        result.errors
    );
}

#[test]
fn logout_request_builds_signs_and_verifies() {
    let key = load_key("dv-signing-1");
    let msg = create_logout_request(
        "transient-id-abc",
        "urn:test:dv",
        "https://rd.example.com/slo",
        &key,
    )
    .expect("LogoutRequest built");

    assert!(msg.xml.contains("LogoutRequest"));
    assert!(
        msg.xml
            .contains("<saml:NameID>transient-id-abc</saml:NameID>")
    );
    let result = verify_xml_signature(&msg.xml, std::slice::from_ref(&key), None);
    assert!(
        result.is_valid(),
        "LogoutRequest signature: {:?}",
        result.errors
    );
}

/// Two distinct signing keys: a message signed by one must NOT verify against
/// the other (verification must not accept an untrusted key).
#[test]
fn message_signed_by_one_key_rejected_by_another() {
    let signer = load_key("dv-signing-1");
    let other = load_key("rd-signing-1");
    let msg = create_authn_request(&AuthnRequestSpec {
        entity_id: "urn:test:dv",
        service_uuid: "uuid-1",
        sso_url: "https://rd.example.com/sso",
        signing_key: &signer,
        preselected_ad_entity_id: None,
        acs_url: None,
    })
    .expect("built");
    let result = verify_xml_signature(&msg.xml, std::slice::from_ref(&other), None);
    assert!(
        !result.is_valid(),
        "must not verify against an unrelated key"
    );
}

/// A Test-mode AuthnRequest carrying an explicit `AssertionConsumerServiceURL`
/// must still sign and verify (the substituted attribute must not break c14n).
#[test]
fn authn_request_with_acs_url_signs_and_verifies() {
    let key = load_key("dv-signing-1");
    let msg = create_authn_request(&AuthnRequestSpec {
        entity_id: "urn:test:dv",
        service_uuid: "uuid-1",
        sso_url: "https://rd.example.com/sso",
        signing_key: &key,
        preselected_ad_entity_id: None,
        acs_url: Some("https://pr-7.preview.example.test/saml/sp/acs"),
    })
    .expect("AuthnRequest built");

    assert!(
        msg.xml.contains(
            "AssertionConsumerServiceURL=\"https://pr-7.preview.example.test/saml/sp/acs\""
        )
    );
    assert!(!msg.xml.contains("AssertionConsumerServiceIndex"));
    let result = verify_xml_signature(&msg.xml, std::slice::from_ref(&key), None);
    assert!(
        result.is_valid(),
        "AuthnRequest signature: {:?}",
        result.errors
    );
}
