//! Fetch and validate real TVS (Toegang Verlening Service) metadata from DICTU.
//!
//! These tests hit external URLs and require network access.
//! Run with: cargo test --test tvs_metadata -- --ignored

use auth_service::saml::{
    constants::{NS_DSIG, NS_MD},
    idp_metadata::extract_idp_keys,
    verification::verify_xml_signature,
    xml_parser::{descendants_by_tag, find_descendant, inner_text},
};

const TVS_PP_METADATA_URL: &str = "https://pp2.toegang.overheid.nl/kvs/rd/metadata";
const TVS_PROD_METADATA_URL: &str = "https://rd2.toegang.overheid.nl/kvs/rd/metadata";

async fn fetch_metadata(url: &str) -> String {
    reqwest::get(url)
        .await
        .unwrap_or_else(|e| panic!("Failed to fetch {url}: {e}"))
        .text()
        .await
        .unwrap_or_else(|e| panic!("Failed to read response from {url}: {e}"))
}

fn validate_metadata(xml: &str, url: &str) {
    let doc = auth_service::saml::xml_parser::parse(xml)
        .unwrap_or_else(|e| panic!("{url}: XML parse error: {e}"));
    let root = doc.document_element();

    // Root must be EntityDescriptor with an entityID
    assert_eq!(
        doc.local_name(root),
        Some("EntityDescriptor"),
        "{url}: root element is not EntityDescriptor"
    );
    assert!(
        doc.get_attribute(root, "entityID").is_some(),
        "{url}: missing entityID attribute"
    );

    // Must contain an IDPSSODescriptor
    let idp = find_descendant(&doc, root, NS_MD, "IDPSSODescriptor")
        .unwrap_or_else(|| panic!("{url}: missing IDPSSODescriptor"));

    // Must expose SingleSignOnService endpoint
    assert!(
        find_descendant(&doc, idp, NS_MD, "SingleSignOnService").is_some(),
        "{url}: missing SingleSignOnService"
    );

    // Must expose ArtifactResolutionService endpoint
    assert!(
        find_descendant(&doc, idp, NS_MD, "ArtifactResolutionService").is_some(),
        "{url}: missing ArtifactResolutionService"
    );

    // Extract keys separated by intended use
    let keys = extract_idp_keys(&doc, root);

    // Must have 1 or 2 signing certificates
    assert!(
        keys.signing.len() == 1 || keys.signing.len() == 2,
        "{url}: expected 1 or 2 signing keys, got {}",
        keys.signing.len()
    );

    // IdP metadata may have 0-2 encryption keys (typically 0: only SPs
    // publish encryption keys so the IdP can encrypt assertions for them)
    assert!(
        keys.encryption.len() <= 2,
        "{url}: expected at most 2 encryption keys, got {}",
        keys.encryption.len()
    );

    // Every KeyDescriptor must have an explicit use attribute; a bare
    // KeyDescriptor (use for both) would be a misconfiguration in TVS
    for kd in descendants_by_tag(&doc, root, NS_MD, "KeyDescriptor") {
        let use_attr = doc.get_attribute(kd, "use");
        assert!(
            use_attr == Some("signing") || use_attr == Some("encryption"),
            "{url}: KeyDescriptor has unexpected use attribute: {use_attr:?}"
        );
    }

    // Metadata must be signed
    let sig = find_descendant(&doc, root, NS_DSIG, "Signature")
        .unwrap_or_else(|| panic!("{url}: metadata is not signed"));

    // TVS metadata signatures use KeyName: verify our derived key_name
    // matches the KeyName in the Signature's KeyInfo
    if let Some(key_name_node) = find_descendant(&doc, sig, NS_DSIG, "KeyName") {
        let sig_key_name = inner_text(&doc, key_name_node);
        let sig_key_name = sig_key_name.trim();
        assert!(
            keys.signing.iter().any(|k| k.key_name == sig_key_name),
            "{url}: Signature KeyName '{sig_key_name}' not found in signing KeyDescriptors"
        );
    }

    // Verify the XML signature using ONLY the signing keys
    let result = verify_xml_signature(xml, &keys.signing, None);
    assert!(
        result.is_valid(),
        "{url}: signature verification with signing keys failed: {:?}",
        result.errors
    );

    // If there are encryption-only keys, they must NOT verify the signature
    let signing_thumbprints: Vec<&str> = keys.signing.iter().map(|k| k.key_name.as_str()).collect();
    let encryption_only: Vec<_> = keys
        .encryption
        .iter()
        .filter(|k| !signing_thumbprints.contains(&k.key_name.as_str()))
        .cloned()
        .collect();

    if !encryption_only.is_empty() {
        let result = verify_xml_signature(xml, &encryption_only, None);
        assert!(
            !result.is_valid(),
            "{url}: signature verification should fail with encryption-only keys"
        );
    }
}

#[tokio::test]
#[ignore] // requires network access; run with: cargo test --test tvs_metadata -- --ignored
async fn validate_preproduction_metadata() {
    let xml = fetch_metadata(TVS_PP_METADATA_URL).await;
    validate_metadata(&xml, TVS_PP_METADATA_URL);
}

#[tokio::test]
#[ignore] // requires network access; run with: cargo test --test tvs_metadata -- --ignored
async fn validate_production_metadata() {
    let xml = fetch_metadata(TVS_PROD_METADATA_URL).await;
    validate_metadata(&xml, TVS_PROD_METADATA_URL);
}
