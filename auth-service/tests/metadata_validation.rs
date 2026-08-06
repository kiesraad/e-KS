//! Integration tests for IdP metadata validation.
//!
//! These tests build signed metadata using our own test certificates, then
//! verify that every validation check correctly rejects malformed input.
//!
//! Run with: cargo test --test metadata_validation

use auth_service::{
    keys::KeyPair,
    saml::{
        constants::{NS_DSIG, NS_MD},
        crypto::sign,
        idp_metadata::extract_idp_keys,
        verification::verify_xml_signature,
        xml_parser::{find_descendant, inner_text},
    },
};
use secrecy::ExposeSecret;

mod common;
use common::{inline_signature, load_key};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Build a minimal IdP metadata document (unsigned).
fn build_idp_metadata(entity_id: &str, id: &str, keys: &[(&str, &str, &str)]) -> String {
    let mut key_descriptors = String::new();
    for (use_attr, key_name, cert_b64) in keys {
        key_descriptors.push_str(&format!(
            r#"<md:KeyDescriptor use="{use_attr}"><ds:KeyInfo xmlns:ds="http://www.w3.org/2000/09/xmldsig#"><ds:KeyName>{key_name}</ds:KeyName><ds:X509Data><ds:X509Certificate>{cert_b64}</ds:X509Certificate></ds:X509Data></ds:KeyInfo></md:KeyDescriptor>"#
        ));
    }
    format!(
        r#"<md:EntityDescriptor xmlns:md="urn:oasis:names:tc:SAML:2.0:metadata" xmlns:ds="http://www.w3.org/2000/09/xmldsig#" ID="{id}" entityID="{entity_id}" cacheDuration="PT24H"><md:IDPSSODescriptor WantAuthnRequestsSigned="true" protocolSupportEnumeration="urn:oasis:names:tc:SAML:2.0:protocol">{key_descriptors}<md:ArtifactResolutionService Binding="urn:oasis:names:tc:SAML:2.0:bindings:SOAP" Location="https://rd.example.com/ars" index="0"/><md:SingleSignOnService Binding="urn:oasis:names:tc:SAML:2.0:bindings:HTTP-POST" Location="https://rd.example.com/sso"/><md:SingleLogoutService Binding="urn:oasis:names:tc:SAML:2.0:bindings:HTTP-POST" Location="https://rd.example.com/slo"/></md:IDPSSODescriptor></md:EntityDescriptor>"#
    )
}

/// Insert the inline signature before `anchor` and sign it in place.
fn sign_inline(xml: &str, id: &str, signing_key: &KeyPair, anchor: &str) -> String {
    let sig = inline_signature(id, &signing_key.cert_base64);
    let pos = xml.find(anchor).expect("signature anchor present");
    let templated = format!("{}{}{}", &xml[..pos], sig, &xml[pos..]);
    sign(&templated, signing_key.key_pem.expose_secret()).expect("signing must succeed")
}

/// Build and sign an IdP metadata document using the given key.
fn signed_metadata(signing_key: &KeyPair) -> String {
    let id = "_test_metadata_001";
    let xml = build_idp_metadata(
        "urn:test:idp",
        id,
        &[("signing", &signing_key.key_name, &signing_key.cert_base64)],
    );
    sign_inline(&xml, id, signing_key, "<md:IDPSSODescriptor")
}

/// Flip the first base64 character inside `<tag>` so the value no longer matches
/// what was signed. Panics if the tag is absent (a broken fixture, not a pass).
fn flip_first_byte_in(xml: &str, tag: &str) -> String {
    let open = format!("<{tag}>");
    let pos = xml
        .find(&open)
        .unwrap_or_else(|| panic!("no {tag} found in signed metadata"));
    let start = pos + open.len();
    let mut out = xml.to_string();
    let c = out.as_bytes()[start];
    let replacement = if c == b'A' { b'B' } else { b'A' };
    // SAFETY: both bytes are ASCII, so the string stays valid UTF-8.
    unsafe { out.as_bytes_mut()[start] = replacement };
    out
}

/// Build and sign metadata with two signing keys (rollover scenario).
fn signed_metadata_with_rollover(key1: &KeyPair, key2: &KeyPair, signing_key: &KeyPair) -> String {
    let id = "_test_metadata_rollover";
    let xml = build_idp_metadata(
        "urn:test:idp",
        id,
        &[
            ("signing", &key1.key_name, &key1.cert_base64),
            ("signing", &key2.key_name, &key2.cert_base64),
        ],
    );
    sign_inline(&xml, id, signing_key, "<md:IDPSSODescriptor")
}

// ---------------------------------------------------------------------------
// Tests: valid metadata
// ---------------------------------------------------------------------------

#[test]
fn valid_signed_metadata_verifies() {
    let key = load_key("rd-signing-1");
    let xml = signed_metadata(&key);

    let result = verify_xml_signature(&xml, &[key], None);
    assert!(
        result.is_valid(),
        "valid metadata must verify: {:?}",
        result.errors
    );
}

#[test]
fn valid_metadata_key_extraction() {
    let key = load_key("rd-signing-1");
    let xml = signed_metadata(&key);

    let doc = auth_service::saml::xml_parser::parse(&xml).unwrap();
    let root = doc.document_element();
    let keys = extract_idp_keys(&doc, root);

    assert_eq!(
        keys.signing.len(),
        1,
        "should extract exactly 1 signing key"
    );
    assert_eq!(keys.encryption.len(), 0, "should extract 0 encryption keys");
    assert_eq!(keys.signing[0].key_name, key.key_name);
}

#[test]
fn valid_metadata_has_correct_structure() {
    let key = load_key("rd-signing-1");
    let xml = signed_metadata(&key);

    let doc = auth_service::saml::xml_parser::parse(&xml).unwrap();
    let root = doc.document_element();
    assert_eq!(doc.local_name(root), Some("EntityDescriptor"));
    assert_eq!(doc.get_attribute(root, "entityID"), Some("urn:test:idp"));
    assert!(doc.get_attribute(root, "ID").is_some());
    assert!(find_descendant(&doc, root, NS_MD, "IDPSSODescriptor").is_some());
    assert!(find_descendant(&doc, root, NS_DSIG, "Signature").is_some());
    assert!(find_descendant(&doc, root, NS_MD, "SingleSignOnService").is_some());
    assert!(find_descendant(&doc, root, NS_MD, "ArtifactResolutionService").is_some());
}

// ---------------------------------------------------------------------------
// Tests: signature validation failures
// ---------------------------------------------------------------------------

#[test]
fn rejects_tampered_entity_id() {
    let key = load_key("rd-signing-1");
    let xml = signed_metadata(&key);
    // Tamper with the entityID (changes the signed content)
    let tampered = xml.replace("urn:test:idp", "urn:test:EVIL");

    let result = verify_xml_signature(&tampered, &[key], None);
    assert!(
        !result.is_valid(),
        "tampered entityID must fail verification"
    );
}

#[test]
fn rejects_tampered_signature_value() {
    let key = load_key("rd-signing-1");
    let xml = signed_metadata(&key);

    let tampered = flip_first_byte_in(&xml, "dsig:SignatureValue");

    let result = verify_xml_signature(&tampered, &[key], None);
    assert!(
        !result.is_valid(),
        "tampered SignatureValue must fail verification"
    );
}

#[test]
fn rejects_tampered_digest_value() {
    let key = load_key("rd-signing-1");
    let xml = signed_metadata(&key);

    let tampered = flip_first_byte_in(&xml, "dsig:DigestValue");

    let result = verify_xml_signature(&tampered, &[key], None);
    assert!(
        !result.is_valid(),
        "tampered DigestValue must fail verification"
    );
}

#[test]
fn rejects_removed_signature() {
    let key = load_key("rd-signing-1");
    let xml = signed_metadata(&key);

    // Strip the dsig:Signature element entirely
    let start = xml.find("<dsig:Signature").expect("must have Signature");
    let end = xml
        .find("</dsig:Signature>")
        .expect("must have closing tag")
        + "</dsig:Signature>".len();
    let stripped = format!("{}{}", &xml[..start], &xml[end..]);

    let result = verify_xml_signature(&stripped, &[key], None);
    assert!(!result.is_valid(), "missing signature must fail");
    assert!(
        result.errors.iter().any(|e| e.contains("No ds:Signature")),
        "error should mention missing signature: {:?}",
        result.errors
    );
}

#[test]
fn rejects_wrong_signing_key() {
    let rd_key = load_key("rd-signing-1");
    let dv_key = load_key("dv-signing-1");
    let xml = signed_metadata(&rd_key);

    // Verify with a different key (DV key instead of RD key)
    let result = verify_xml_signature(&xml, &[dv_key], None);
    assert!(!result.is_valid(), "wrong key must fail verification");
}

#[test]
fn rejects_mismatched_cert_base64() {
    let rd_key = load_key("rd-signing-1");
    let xml = signed_metadata(&rd_key);

    // Our signature template embeds X509Certificate, not KeyName.
    // Providing a trusted key with a different cert_base64 must fail.
    let mut wrong = rd_key.clone();
    wrong.cert_base64 = "AAAA".to_string();
    wrong.key_name = "0000000000000000000000000000000000000000".to_string();

    let result = verify_xml_signature(&xml, &[wrong], None);
    assert!(!result.is_valid(), "mismatched cert must fail");
    assert!(
        result.errors.iter().any(|e| e.contains("does not match")),
        "error should mention cert mismatch: {:?}",
        result.errors
    );
}

#[test]
fn rejects_empty_trusted_keys() {
    let key = load_key("rd-signing-1");
    let xml = signed_metadata(&key);

    // No trusted keys at all
    let result = verify_xml_signature(&xml, &[], None);
    assert!(!result.is_valid(), "empty trust store must fail");
}

// ---------------------------------------------------------------------------
// Tests: key extraction edge cases
// ---------------------------------------------------------------------------

#[test]
fn key_extraction_bare_key_descriptor_goes_to_both() {
    // A KeyDescriptor without a `use` attribute should be usable for both signing and encryption.
    let key = load_key("rd-signing-1");
    let xml = format!(
        r#"<md:EntityDescriptor xmlns:md="urn:oasis:names:tc:SAML:2.0:metadata" xmlns:ds="http://www.w3.org/2000/09/xmldsig#" entityID="urn:test">
<md:IDPSSODescriptor protocolSupportEnumeration="urn:oasis:names:tc:SAML:2.0:protocol">
<md:KeyDescriptor><ds:KeyInfo><ds:X509Data><ds:X509Certificate>{}</ds:X509Certificate></ds:X509Data></ds:KeyInfo></md:KeyDescriptor>
</md:IDPSSODescriptor></md:EntityDescriptor>"#,
        key.cert_base64
    );
    let doc = auth_service::saml::xml_parser::parse(&xml).unwrap();
    let root = doc.document_element();
    let keys = extract_idp_keys(&doc, root);

    assert_eq!(keys.signing.len(), 1, "bare key must appear in signing");
    assert_eq!(
        keys.encryption.len(),
        1,
        "bare key must appear in encryption"
    );
    assert_eq!(keys.signing[0].key_name, keys.encryption[0].key_name);
}

#[test]
fn key_extraction_signing_only_not_in_encryption() {
    let key = load_key("rd-signing-1");
    let xml = format!(
        r#"<md:EntityDescriptor xmlns:md="urn:oasis:names:tc:SAML:2.0:metadata" xmlns:ds="http://www.w3.org/2000/09/xmldsig#" entityID="urn:test">
<md:IDPSSODescriptor protocolSupportEnumeration="urn:oasis:names:tc:SAML:2.0:protocol">
<md:KeyDescriptor use="signing"><ds:KeyInfo><ds:X509Data><ds:X509Certificate>{}</ds:X509Certificate></ds:X509Data></ds:KeyInfo></md:KeyDescriptor>
</md:IDPSSODescriptor></md:EntityDescriptor>"#,
        key.cert_base64
    );
    let doc = auth_service::saml::xml_parser::parse(&xml).unwrap();
    let root = doc.document_element();
    let keys = extract_idp_keys(&doc, root);

    assert_eq!(keys.signing.len(), 1);
    assert_eq!(
        keys.encryption.len(),
        0,
        "signing key must not appear in encryption"
    );
}

#[test]
fn key_extraction_encryption_only_not_in_signing() {
    let xml = format!(
        r#"<md:EntityDescriptor xmlns:md="urn:oasis:names:tc:SAML:2.0:metadata" xmlns:ds="http://www.w3.org/2000/09/xmldsig#" entityID="urn:test">
<md:IDPSSODescriptor protocolSupportEnumeration="urn:oasis:names:tc:SAML:2.0:protocol">
<md:KeyDescriptor use="encryption"><ds:KeyInfo><ds:X509Data><ds:X509Certificate>{}</ds:X509Certificate></ds:X509Data></ds:KeyInfo></md:KeyDescriptor>
</md:IDPSSODescriptor></md:EntityDescriptor>"#,
        load_key("dv-encryption-1").cert_base64
    );
    let doc = auth_service::saml::xml_parser::parse(&xml).unwrap();
    let root = doc.document_element();
    let keys = extract_idp_keys(&doc, root);

    assert_eq!(
        keys.signing.len(),
        0,
        "encryption key must not appear in signing"
    );
    assert_eq!(keys.encryption.len(), 1);
}

#[test]
fn key_extraction_skips_key_name_only_descriptors() {
    // KeyDescriptor with KeyName but no X509Certificate should be skipped
    let xml = r#"<md:EntityDescriptor xmlns:md="urn:oasis:names:tc:SAML:2.0:metadata" xmlns:ds="http://www.w3.org/2000/09/xmldsig#" entityID="urn:test">
<md:IDPSSODescriptor protocolSupportEnumeration="urn:oasis:names:tc:SAML:2.0:protocol">
<md:KeyDescriptor use="signing"><ds:KeyInfo><ds:KeyName>some-key-name</ds:KeyName></ds:KeyInfo></md:KeyDescriptor>
</md:IDPSSODescriptor></md:EntityDescriptor>"#;
    let doc = auth_service::saml::xml_parser::parse(xml).unwrap();
    let root = doc.document_element();
    let keys = extract_idp_keys(&doc, root);

    assert_eq!(
        keys.signing.len(),
        0,
        "KeyName-only descriptors must be skipped"
    );
}

#[test]
fn key_extraction_handles_empty_metadata() {
    let xml = r#"<md:EntityDescriptor xmlns:md="urn:oasis:names:tc:SAML:2.0:metadata" entityID="urn:test">
<md:IDPSSODescriptor protocolSupportEnumeration="urn:oasis:names:tc:SAML:2.0:protocol">
</md:IDPSSODescriptor></md:EntityDescriptor>"#;
    let doc = auth_service::saml::xml_parser::parse(xml).unwrap();
    let root = doc.document_element();
    let keys = extract_idp_keys(&doc, root);

    assert_eq!(keys.signing.len(), 0);
    assert_eq!(keys.encryption.len(), 0);
}

// ---------------------------------------------------------------------------
// Tests: certificate rollover
// ---------------------------------------------------------------------------

#[test]
fn rollover_metadata_contains_both_keys() {
    let key1 = load_key("rd-signing-1");
    let key2 = load_key("dv-signing-1"); // use DV key as second signing key for testing
    let xml = signed_metadata_with_rollover(&key1, &key2, &key1);

    let doc = auth_service::saml::xml_parser::parse(&xml).unwrap();
    let root = doc.document_element();
    let keys = extract_idp_keys(&doc, root);

    assert_eq!(keys.signing.len(), 2, "should extract both signing keys");
    let names: Vec<&str> = keys.signing.iter().map(|k| k.key_name.as_str()).collect();
    assert!(names.contains(&key1.key_name.as_str()));
    assert!(names.contains(&key2.key_name.as_str()));
}

#[test]
fn rollover_metadata_verifies_with_either_advertised_key() {
    let key1 = load_key("rd-signing-1");
    let key2 = load_key("dv-signing-1");

    // Whichever of the two advertised keys actually signed, verification must
    // succeed: that is the point of publishing both during a rollover.
    for signer in [&key1, &key2] {
        let xml = signed_metadata_with_rollover(&key1, &key2, signer);
        let doc = auth_service::saml::xml_parser::parse(&xml).unwrap();
        let root = doc.document_element();
        let keys = extract_idp_keys(&doc, root);
        let result = verify_xml_signature(&xml, &keys.signing, None);
        assert!(
            result.is_valid(),
            "rollover metadata signed by {} must verify: {:?}",
            signer.key_name,
            result.errors
        );
    }
}

#[test]
fn rollover_old_key_removed_after_update() {
    let key1 = load_key("rd-signing-1");
    let key2 = load_key("dv-signing-1");

    // First metadata has key1 + key2
    let xml1 = signed_metadata_with_rollover(&key1, &key2, &key1);
    let doc1 = auth_service::saml::xml_parser::parse(&xml1).unwrap();
    let root1 = doc1.document_element();
    let keys1 = extract_idp_keys(&doc1, root1);
    assert_eq!(keys1.signing.len(), 2);

    // Second metadata has only key1 (key2 removed after rollover)
    let xml2 = signed_metadata(&key1);
    let doc2 = auth_service::saml::xml_parser::parse(&xml2).unwrap();
    let root2 = doc2.document_element();
    let keys2 = extract_idp_keys(&doc2, root2);
    assert_eq!(keys2.signing.len(), 1, "old rollover key should be gone");
    assert_eq!(keys2.signing[0].key_name, key1.key_name);
}

// ---------------------------------------------------------------------------
// Tests: key name derivation consistency
// ---------------------------------------------------------------------------

#[test]
fn extracted_key_name_matches_derived_key_name() {
    let key = load_key("rd-signing-1");
    let xml = signed_metadata(&key);

    let doc = auth_service::saml::xml_parser::parse(&xml).unwrap();
    let root = doc.document_element();
    let keys = extract_idp_keys(&doc, root);

    // The key_name derived from the extracted PEM must match the original
    assert_eq!(
        keys.signing[0].key_name, key.key_name,
        "extracted key_name must match original"
    );
}

#[test]
fn signature_x509cert_matches_metadata_key() {
    let key = load_key("rd-signing-1");
    let xml = signed_metadata(&key);

    let doc = auth_service::saml::xml_parser::parse(&xml).unwrap();
    let root = doc.document_element();
    let sig = find_descendant(&doc, root, NS_DSIG, "Signature").expect("must have Signature");
    let x509_node = find_descendant(&doc, sig, NS_DSIG, "X509Certificate")
        .expect("Signature must have X509Certificate");
    let sig_cert: String = inner_text(&doc, x509_node)
        .chars()
        .filter(|c: &char| !c.is_whitespace())
        .collect();

    let keys = extract_idp_keys(&doc, root);
    assert!(
        keys.signing.iter().any(|k| k.cert_base64 == sig_cert),
        "Signature X509Certificate must match a signing KeyDescriptor"
    );
}

// ---------------------------------------------------------------------------
// Tests: XML parsing edge cases
// ---------------------------------------------------------------------------

#[test]
fn rejects_non_xml_input() {
    let result = verify_xml_signature("this is not XML", &[], None);
    assert!(!result.is_valid());
    assert!(result.errors.iter().any(|e| e.contains("parse error")));
}

#[test]
fn rejects_xml_without_signature() {
    let xml = r#"<md:EntityDescriptor xmlns:md="urn:oasis:names:tc:SAML:2.0:metadata" entityID="urn:test" ID="_1">
<md:IDPSSODescriptor protocolSupportEnumeration="urn:oasis:names:tc:SAML:2.0:protocol">
</md:IDPSSODescriptor></md:EntityDescriptor>"#;

    let key = load_key("rd-signing-1");
    let result = verify_xml_signature(xml, &[key], None);
    assert!(!result.is_valid());
    assert!(
        result.errors.iter().any(|e| e.contains("No ds:Signature")),
        "must report missing signature: {:?}",
        result.errors
    );
}

#[test]
fn rejects_tampered_key_descriptor_cert() {
    let key = load_key("rd-signing-1");
    let xml = signed_metadata(&key);

    // Tamper with the X509Certificate in the KeyDescriptor (not the Signature).
    // This changes the trusted cert that would be used for verification.
    let tampered = xml.replacen(&key.cert_base64[..20], "AAAAAAAAAAAAAAAAAAAAAA", 1);

    // Re-extract keys from the tampered metadata; the cert is now different.
    let doc = auth_service::saml::xml_parser::parse(&tampered).unwrap();
    let root = doc.document_element();
    let tampered_keys = extract_idp_keys(&doc, root);

    assert!(
        !tampered_keys.signing.is_empty(),
        "the tampered cert must still be extractable, so the assertion below \
         actually exercises signature verification"
    );

    // The tampered cert no longer matches the genuine signature.
    let result = verify_xml_signature(&tampered, &tampered_keys.signing, None);
    assert!(
        !result.is_valid(),
        "tampered KeyDescriptor cert should fail verification"
    );
}

// ---------------------------------------------------------------------------
// Tests: verify with X509Certificate matching (not KeyName)
// ---------------------------------------------------------------------------

#[test]
fn verifies_via_x509_certificate_matching() {
    let key = load_key("rd-signing-1");
    // Our signature template embeds X509Certificate, not KeyName.
    // A trusted key with matching cert_base64 but empty key_name should still verify
    // via the X509Certificate matching path.
    let trust = KeyPair {
        cert_pem: key.cert_pem.clone(),
        key_pem: String::new().into(),
        key_name: String::new(), // won't match by KeyName, falls through to cert matching
        cert_base64: key.cert_base64.clone(),
    };

    let xml = signed_metadata(&key);
    let result = verify_xml_signature(&xml, &[trust], None);
    assert!(
        result.is_valid(),
        "X509Certificate matching should verify: {:?}",
        result.errors
    );
}
