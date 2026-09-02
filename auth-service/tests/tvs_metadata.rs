//! Fetch and validate real TVS (Toegang Verlening Service) metadata from DICTU.
//!
//! These tests hit external URLs and require network access.
//! Run with: cargo test --test tvs_metadata -- --ignored

use auth_service::saml::{
    constants::{NS_DSIG, NS_MD},
    idp_metadata::{IdpKeys, extract_idp_keys},
    verification::{ExpectedRoot, verify_xml_signature},
    xml_parser::{Document, NodeId, descendants_by_tag, find_descendant, inner_text, parse},
};

/// The root of any SAML metadata document; the `@ID` is the RD's own and is not
/// asserted here.
fn entity_descriptor_root() -> ExpectedRoot<'static> {
    ExpectedRoot {
        namespace: NS_MD,
        local_name: "EntityDescriptor",
        id: None,
    }
}

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
    let doc = parse(xml).unwrap_or_else(|e| panic!("{url}: XML parse error: {e}"));
    let root = doc.document_element();

    validate_structure(&doc, root, url);
    let keys = validate_key_descriptors(&doc, root, url);
    validate_signature(xml, &doc, root, url, &keys);
}

/// The document shape eID §8.4 requires of RD metadata: an `EntityDescriptor`
/// root with an `entityID`, an `IDPSSODescriptor`, and the SSO and artifact
/// resolution endpoints inside that role descriptor.
fn validate_structure(doc: &Document, root: NodeId, url: &str) {
    assert_eq!(
        doc.local_name(root),
        Some("EntityDescriptor"),
        "{url}: root element is not EntityDescriptor"
    );
    assert!(
        doc.get_attribute(root, "entityID").is_some(),
        "{url}: missing entityID attribute"
    );

    let idp = find_descendant(doc, root, NS_MD, "IDPSSODescriptor")
        .unwrap_or_else(|| panic!("{url}: missing IDPSSODescriptor"));
    assert!(
        find_descendant(doc, idp, NS_MD, "SingleSignOnService").is_some(),
        "{url}: missing SingleSignOnService"
    );
    assert!(
        find_descendant(doc, idp, NS_MD, "ArtifactResolutionService").is_some(),
        "{url}: missing ArtifactResolutionService"
    );
}

/// The published key material: the expected counts per use, and an explicit
/// `use` attribute on every `KeyDescriptor` (a bare one, usable for both, would
/// be a TVS misconfiguration).
fn validate_key_descriptors(doc: &Document, root: NodeId, url: &str) -> IdpKeys {
    let keys = extract_idp_keys(doc, root);

    assert!(
        keys.signing.len() == 1 || keys.signing.len() == 2,
        "{url}: expected 1 or 2 signing keys, got {}",
        keys.signing.len()
    );
    // IdP metadata may have 0-2 encryption keys (typically 0: only SPs publish
    // encryption keys so the IdP can encrypt assertions for them).
    assert!(
        keys.encryption.len() <= 2,
        "{url}: expected at most 2 encryption keys, got {}",
        keys.encryption.len()
    );

    for kd in descendants_by_tag(doc, root, NS_MD, "KeyDescriptor") {
        let use_attr = doc.get_attribute(kd, "use");
        assert!(
            use_attr == Some("signing") || use_attr == Some("encryption"),
            "{url}: KeyDescriptor has unexpected use attribute: {use_attr:?}"
        );
    }
    keys
}

/// The metadata signature: present, referencing one of the published signing
/// certs by `KeyName`, verifying against the signing keys, and **not** verifying
/// against an encryption-only key.
fn validate_signature(xml: &str, doc: &Document, root: NodeId, url: &str, keys: &IdpKeys) {
    let sig = find_descendant(doc, root, NS_DSIG, "Signature")
        .unwrap_or_else(|| panic!("{url}: metadata is not signed"));

    // TVS metadata signatures use KeyName: the thumbprint we derive from a
    // published cert must match the KeyName in the Signature's KeyInfo.
    if let Some(key_name_node) = find_descendant(doc, sig, NS_DSIG, "KeyName") {
        let sig_key_name = inner_text(doc, key_name_node).unwrap_or_default();
        let sig_key_name = sig_key_name.trim();
        assert!(
            keys.signing
                .iter()
                .any(|k| k.matches_key_name(sig_key_name)),
            "{url}: Signature KeyName '{sig_key_name}' not found in signing KeyDescriptors"
        );
    }

    let result = verify_xml_signature(xml, &keys.signing, &entity_descriptor_root());
    assert!(
        result.is_valid(),
        "{url}: signature verification with signing keys failed: {:?}",
        result.errors
    );

    // An encryption-only key must never verify the signature (eID §9.2).
    let signing_thumbprints: Vec<&str> = keys.signing.iter().map(|k| k.key_name.as_str()).collect();
    let encryption_only: Vec<_> = keys
        .encryption
        .iter()
        .filter(|k| !signing_thumbprints.contains(&k.key_name.as_str()))
        .cloned()
        .collect();
    if !encryption_only.is_empty() {
        let result = verify_xml_signature(xml, &encryption_only, &entity_descriptor_root());
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
