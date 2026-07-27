//! XML-DSig signature verification (crypto delegated to the [`crypto`]
//! backend).
//!
//! eID §9.2: verification certs MUST come from verified metadata; the KeyInfo's
//! KeyName/X509Certificate only selects which trusted cert to use. So: find the
//! KeyInfo, match it against `trusted_keys`, then verify with that cert.
use crate::{
    keys::KeyPair,
    saml::{
        constants::NS_DSIG,
        crypto::{self, SignatureVerification},
        xml_parser::{
            Document, NodeId, children_by_tag, descendants_by_tag, find_descendant, inner_text,
        },
    },
};
use tracing::debug;

// eID §9.1: SHA-1 is no longer supported (except as the RSA padding hash). The
// SignatureValue MUST use at least RSA-SHA256 and digests at least SHA-256, so we
// only accept the §9.1 algorithm set on incoming signatures and reject anything
// else (notably an rsa-sha1 / sha1 downgrade).
const ALLOWED_SIGNATURE_METHODS: &[&str] = &[
    "http://www.w3.org/2001/04/xmldsig-more#rsa-sha256",
    "http://www.w3.org/2001/04/xmldsig-more#rsa-sha384",
    "http://www.w3.org/2001/04/xmldsig-more#rsa-sha512",
];
// eID §9.1 lists one URI per digest, but its SHA-384/SHA-512 spellings differ
// from the W3C-registered ones (it gives `xmldsig-more#sha512`, the registered
// form is `xmlenc#sha512`). Both spellings of each are accepted so a conformant
// signer is not rejected over a URI alias; the algorithm strength is identical.
const ALLOWED_DIGEST_METHODS: &[&str] = &[
    "http://www.w3.org/2001/04/xmlenc#sha256",
    "http://www.w3.org/2001/04/xmldsig-more#sha384",
    "http://www.w3.org/2001/04/xmlenc#sha384",
    "http://www.w3.org/2001/04/xmlenc#sha512",
    "http://www.w3.org/2001/04/xmldsig-more#sha512",
];

// eID §9.1: "Canonicalization MUST be carried out according to the exclusive
// c14n method without comments". Pinning this matters for more than tidiness: a
// `WithComments` canonicalization would pull comment nodes into the digest, so
// the bytes the signature covers would stop matching the comment-free view the
// validators navigate (see `xml_parser`), which is exactly the gap the XSW
// comment-injection tests probe.
const EXCLUSIVE_C14N: &str = "http://www.w3.org/2001/10/xml-exc-c14n#";

// eID §9.1: the signature is embedded with the Enveloped Signature Transform, so
// every Reference must declare it. Without it the digest would cover the
// `<Signature>` element itself, which cannot be what a valid enveloped signature
// over our message root did.
const ENVELOPED_SIGNATURE_TRANSFORM: &str = "http://www.w3.org/2000/09/xmldsig#enveloped-signature";

/// Outcome of signature verification.
pub struct VerifyResult {
    pub errors: Vec<String>,
}

impl VerifyResult {
    /// Valid exactly when no error was recorded.
    pub fn is_valid(&self) -> bool {
        self.errors.is_empty()
    }
}

/// Verify the enveloping XML-DSig signature of `xml` against `trusted_keys`.
pub fn verify_xml_signature(xml: &str, trusted_keys: &[KeyPair]) -> VerifyResult {
    debug!(
        "[verify] Verifying XML signature (xml_len={}, trusted_keys={})",
        xml.len(),
        trusted_keys.len()
    );
    let mut errors = Vec::new();

    // Parse the XML once; the document is used for key matching, signature
    // enumeration, algorithm checks and the Reference-covers-root check.
    let doc = match crate::saml::xml_parser::parse(xml) {
        Ok(d) => d,
        Err(e) => {
            return VerifyResult {
                errors: vec![format!("XML parse error: {e}")],
            };
        }
    };
    let root = doc.document_element();

    let sig_node = match enveloping_signature(&doc, root) {
        Ok(n) => n,
        Err(e) => return VerifyResult { errors: vec![e] },
    };
    'verify: {
        let Some(cert) = find_matching_key(&doc, sig_node, trusted_keys, &mut errors) else {
            break 'verify;
        };
        debug!(
            "[verify] Signature matched trusted key key_name={}",
            cert.key_name
        );

        // eID §9.1: reject weak (e.g. SHA-1) signature/digest algorithms before
        // trusting the signature, regardless of what the backend would accept.
        if !check_signature_algorithms(&doc, sig_node, &mut errors) {
            break 'verify;
        }

        // SECURITY (XSW): bind "signature valid" to "this root element was
        // signed". The backend only checks that each Reference's digest matches
        // its resolved target; in a detached layout that target may be a sibling
        // of the Signature while we consume the root. eID uses one enveloped
        // signature over the message root, so require every Reference to target
        // this root (empty URI = whole document, or "#<root-id>"). Duplicate IDs
        // are rejected by the backend, so a matching ID uniquely names the root.
        if !signature_covers_root(&doc, root, sig_node, &mut errors) {
            break 'verify;
        }

        match crypto::verify_signature(xml, &cert.cert_pem) {
            Ok(SignatureVerification::Valid) => {
                debug!("[verify] Signature OK");
            }
            Ok(SignatureVerification::Invalid(reason)) => {
                errors.push(format!("Signature verification failed: {reason}"));
            }
            Err(e) => {
                errors.push(e.to_string());
            }
        }
    }

    debug!(
        "[verify] Signature verification done: valid={}, errors={}",
        errors.is_empty(),
        errors.len()
    );
    VerifyResult { errors }
}

/// Locate the single *enveloping* `<Signature>` (a direct child of `root`) and
/// require it to be the first `<Signature>` in the whole document; a structural
/// violation is returned as the error message.
///
/// eID §7.6.1/§7.6.3: verify only the *enveloping* signature. Nested signatures
/// belong to nested elements signed by a different party (e.g. the DigiD AD
/// inside an ArtifactResponse), so checking them against the RD keys would
/// wrongly reject the response. The backend verifies the first signature it
/// finds, so more than one enveloping signature is ambiguous. eID messages
/// carry exactly one.
///
/// SECURITY (XSW): the crypto backend verifies the *first* `<Signature>` in
/// document order, which is not necessarily the enveloping one. A nested
/// signature placed earlier in the document (e.g. a genuine RD signature
/// wrapped inside a forged root) would be the signature actually verified,
/// while the structural checks only inspect the enveloping signature, so a
/// bogus never-verified enveloping signature could authenticate a forged root.
/// Require the enveloping signature to be the first `<Signature>` in the whole
/// document so the backend verifies exactly the signature we authorize.
/// Genuine eID messages always place it first (Issuer -> Signature ->
/// Status/Response), so this only rejects wrapped documents.
fn enveloping_signature(doc: &Document, root: NodeId) -> Result<NodeId, String> {
    let sig_nodes = children_by_tag(doc, root, NS_DSIG, "Signature");
    debug!(
        "[verify] Found {} enveloping Signature element(s) on <{}>",
        sig_nodes.len(),
        doc.local_name(root).unwrap_or_default()
    );
    if sig_nodes.is_empty() {
        return Err("No ds:Signature element found".to_string());
    }
    if sig_nodes.len() > 1 {
        return Err(format!(
            "Expected exactly one enveloping ds:Signature, found {}",
            sig_nodes.len()
        ));
    }

    let all_sigs = descendants_by_tag(doc, root, NS_DSIG, "Signature");
    if all_sigs.first() != sig_nodes.first() {
        return Err(
            "A nested ds:Signature precedes the enveloping signature: the backend would \
             verify a different signature than the enveloping one (possible XML signature \
             wrapping)"
                .to_string(),
        );
    }
    Ok(sig_nodes[0])
}

// eID §9.1: verify the Signature's declared SignatureMethod, every Reference
// DigestMethod, the canonicalization method and the Reference transforms are in
// the permitted set. Returns `false` (and records an error) on any disallowed
// or missing algorithm so a weak signature is not trusted even if the crypto
// backend could verify it.
fn check_signature_algorithms(doc: &Document, sig: NodeId, errors: &mut Vec<String>) -> bool {
    // Evaluate all four so every violation is reported, not just the first.
    let sig_method_ok = check_signature_method(doc, sig, errors);
    let digests_ok = check_digest_methods(doc, sig, errors);
    let c14n_ok = check_canonicalization_method(doc, sig, errors);
    let transforms_ok = check_reference_transforms(doc, sig, errors);
    sig_method_ok && digests_ok && c14n_ok && transforms_ok
}

// eID §9.1: the SignatureMethod MUST be RSA-SHA256 or stronger (no SHA-1).
fn check_signature_method(doc: &Document, sig: NodeId, errors: &mut Vec<String>) -> bool {
    match find_descendant(doc, sig, NS_DSIG, "SignatureMethod")
        .and_then(|n| doc.get_attribute(n, "Algorithm"))
    {
        Some(a) if ALLOWED_SIGNATURE_METHODS.contains(&a) => true,
        Some(a) => {
            errors.push(format!(
                "Disallowed SignatureMethod (eID §9.1 requires RSA-SHA256 or stronger): {a}"
            ));
            false
        }
        None => {
            errors.push("Signature has no SignatureMethod algorithm".to_string());
            false
        }
    }
}

// eID §9.1: every Reference DigestMethod MUST be SHA-256 or stronger.
fn check_digest_methods(doc: &Document, sig: NodeId, errors: &mut Vec<String>) -> bool {
    let mut ok = true;
    let digests = descendants_by_tag(doc, sig, NS_DSIG, "DigestMethod");
    if digests.is_empty() {
        errors.push("Signature has no DigestMethod".to_string());
        ok = false;
    }
    for d in digests {
        match doc.get_attribute(d, "Algorithm") {
            Some(a) if ALLOWED_DIGEST_METHODS.contains(&a) => {}
            Some(a) => {
                errors.push(format!(
                    "Disallowed DigestMethod (eID §9.1 requires SHA-256 or stronger): {a}"
                ));
                ok = false;
            }
            None => {
                errors.push("DigestMethod has no Algorithm".to_string());
                ok = false;
            }
        }
    }
    ok
}

// eID §9.1: exclusive c14n without comments, on the SignedInfo itself.
fn check_canonicalization_method(doc: &Document, sig: NodeId, errors: &mut Vec<String>) -> bool {
    match find_descendant(doc, sig, NS_DSIG, "CanonicalizationMethod")
        .and_then(|n| doc.get_attribute(n, "Algorithm"))
    {
        Some(EXCLUSIVE_C14N) => true,
        Some(a) => {
            errors.push(format!(
                "Disallowed CanonicalizationMethod (eID §9.1 requires exclusive c14n \
                 without comments): {a}"
            ));
            false
        }
        None => {
            errors.push("Signature has no CanonicalizationMethod algorithm".to_string());
            false
        }
    }
}

// eID §9.1: every Reference must apply the enveloped-signature transform, and
// any c14n transform it names must also be the exclusive, comment-free one.
fn check_reference_transforms(doc: &Document, sig: NodeId, errors: &mut Vec<String>) -> bool {
    let mut ok = true;
    for r in descendants_by_tag(doc, sig, NS_DSIG, "Reference") {
        let transforms: Vec<&str> = descendants_by_tag(doc, r, NS_DSIG, "Transform")
            .into_iter()
            .filter_map(|t| doc.get_attribute(t, "Algorithm"))
            .collect();
        if !transforms.contains(&ENVELOPED_SIGNATURE_TRANSFORM) {
            errors.push(
                "Signature Reference does not apply the enveloped-signature transform \
                 (eID §9.1)"
                    .to_string(),
            );
            ok = false;
        }
        if let Some(bad) = transforms
            .iter()
            .find(|t| t.contains("c14n") && **t != EXCLUSIVE_C14N)
        {
            errors.push(format!(
                "Signature Reference uses a disallowed canonicalization transform \
                 (eID §9.1 requires exclusive c14n without comments): {bad}"
            ));
            ok = false;
        }
    }
    ok
}

// SECURITY (XSW): every `<Reference>` of the enveloping signature MUST target
// the root element `root` (the element whose data the caller consumes). Accepts
// an empty URI (whole document) or `#<id>` where `<id>` is the root's ID
// attribute. Returns `false` (and records an error) on a missing or off-root
// reference, so a signature whose digest matches a sibling/nested element cannot
// authenticate a forged root wrapped around it.
fn signature_covers_root(
    doc: &Document,
    root: NodeId,
    sig: NodeId,
    errors: &mut Vec<String>,
) -> bool {
    let root_id = ["ID", "Id", "id", "AssertionID"]
        .iter()
        .find_map(|a| doc.get_attribute(root, a));

    let refs = descendants_by_tag(doc, sig, NS_DSIG, "Reference");
    if refs.is_empty() {
        errors.push("Signature has no Reference".to_string());
        return false;
    }
    for r in refs {
        let uri = doc.get_attribute(r, "URI").unwrap_or("");
        let targets_root =
            uri.is_empty() || root_id.is_some_and(|id| uri.strip_prefix('#') == Some(id));
        if !targets_root {
            errors.push(format!(
                "Signature Reference URI {uri:?} does not target the signed root element \
                 (possible XML signature wrapping)"
            ));
            return false;
        }
    }
    true
}

fn find_matching_key<'a>(
    doc: &Document,
    sig: NodeId,
    trusted: &'a [KeyPair],
    errors: &mut Vec<String>,
) -> Option<&'a KeyPair> {
    if let Some(key_name_node) = find_descendant(doc, sig, NS_DSIG, "KeyName") {
        let key_name = inner_text(doc, key_name_node);
        let key_name = key_name.trim();
        if let Some(found) = trusted.iter().find(|kp| kp.matches_key_name(key_name)) {
            return Some(found);
        }
        let trusted_names: Vec<&str> = trusted.iter().map(|kp| kp.key_name.as_str()).collect();
        errors.push(format!(
            "Unknown KeyName in signature: {key_name} (trusted: {trusted_names:?})"
        ));
        return None;
    }

    if let Some(x509_node) = find_descendant(doc, sig, NS_DSIG, "X509Certificate") {
        let sig_cert = inner_text(doc, x509_node).replace(|c: char| c.is_whitespace(), "");
        if let Some(found) = trusted.iter().find(|kp| kp.cert_base64 == sig_cert) {
            return Some(found);
        }
        errors.push("X509Certificate in signature does not match any trusted key".to_string());
        return None;
    }

    errors.push("Signature KeyInfo contains neither KeyName nor X509Certificate".to_string());
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    fn key_pair(cert_pem: &str) -> KeyPair {
        KeyPair::from_pem(cert_pem.to_string(), String::new().into())
    }

    // A self-contained test certificate (never used for real signing).
    const TEST_PEM: &str = "-----BEGIN CERTIFICATE-----\nMIIBkTCB+wIJALRiMLAh0WNHMA0GCSqGSIb3DQEBCwUAMBExDzANBgNVBAMMBnRlc3RDQTAeFw0yNTAxMDEwMDAwMDBaFw0yNjAxMDEwMDAwMDBaMBExDzANBgNVBAMMBnRlc3RDQTBcMA0GCSqGSIb3DQEBAQUAA0sAMEgCQQC7o96P+5MhMjCnSGfnMhKxGdzQ7vNvPJGK7eCRvig6V7l6x2mBOFp2Z9gE4yrGS0ISjqRIG1WQ5rOb3Uz9AgMBAAEwDQYJKoZIhvcNAQELBQADQQBb+u1uAt6HlG7MFQHtWJ0RI0U8C/XIfDqFa7OmGjPGqEjNdvTY3Zll8TfhUPGCNjBHPkTO1LjI/mO07m7bZO4\n-----END CERTIFICATE-----";

    #[test]
    fn only_the_root_signature_is_considered() {
        // A Signature nested in a child but none enveloping the root: the
        // nested one (e.g. an Assertion's, signed by a different party) must be
        // ignored here so it doesn't fail ArtifactResponse verification.
        let xml = r#"<ArtifactResponse><Status/><Response><Signature><KeyInfo><KeyName>x</KeyName></KeyInfo></Signature></Response></ArtifactResponse>"#;
        let result = verify_xml_signature(xml, &[key_pair(TEST_PEM)]);
        assert!(!result.is_valid());
        assert_eq!(
            result.errors,
            vec!["No ds:Signature element found".to_string()]
        );
    }

    #[test]
    fn find_matching_key_matches_sha256_key_name() {
        let kp = key_pair(TEST_PEM);
        let sha256 = crate::keys::derive_key_names(TEST_PEM)[1].clone();
        let xml = format!(
            r#"<Signature xmlns="http://www.w3.org/2000/09/xmldsig#"><KeyInfo><KeyName>{sha256}</KeyName></KeyInfo></Signature>"#
        );
        let doc = crate::saml::xml_parser::parse(&xml).unwrap();
        let sig = doc.document_element();
        let mut errors = Vec::new();
        let found = find_matching_key(&doc, sig, std::slice::from_ref(&kp), &mut errors);
        assert!(found.is_some(), "SHA-256 KeyName should match: {errors:?}");
    }

    /// A standalone `<Signature>` carrying just the algorithm declarations
    /// `check_signature_algorithms` inspects.
    fn signed_info(sig_alg: &str, digest_alg: &str, c14n: &str, transforms: &[&str]) -> String {
        let transforms: String = transforms
            .iter()
            .map(|t| format!(r#"<Transform Algorithm="{t}"/>"#))
            .collect();
        format!(
            r#"<Signature xmlns="{NS_DSIG}"><SignedInfo><CanonicalizationMethod Algorithm="{c14n}"/><SignatureMethod Algorithm="{sig_alg}"/><Reference><Transforms>{transforms}</Transforms><DigestMethod Algorithm="{digest_alg}"/></Reference></SignedInfo></Signature>"#
        )
    }

    fn algorithm_errors(xml: &str) -> (bool, Vec<String>) {
        let doc = crate::saml::xml_parser::parse(xml).unwrap();
        let sig = doc.document_element();
        let mut errors = Vec::new();
        let ok = check_signature_algorithms(&doc, sig, &mut errors);
        (ok, errors)
    }

    #[test]
    fn check_signature_algorithms_requires_exclusive_c14n() {
        // eID §9.1: exclusive c14n WITHOUT comments. The WithComments variant
        // would pull comments into the digest, breaking the equivalence between
        // what is signed and the comment-free tree the validators read.
        let (ok, errors) = algorithm_errors(&signed_info(
            "http://www.w3.org/2001/04/xmldsig-more#rsa-sha256",
            "http://www.w3.org/2001/04/xmlenc#sha256",
            "http://www.w3.org/2001/10/xml-exc-c14n#WithComments",
            &[ENVELOPED_SIGNATURE_TRANSFORM, EXCLUSIVE_C14N],
        ));
        assert!(!ok);
        assert!(
            errors
                .iter()
                .any(|e| e.contains("Disallowed CanonicalizationMethod")),
            "{errors:?}"
        );

        // Inclusive c14n named as a Reference transform is rejected too.
        let (ok, errors) = algorithm_errors(&signed_info(
            "http://www.w3.org/2001/04/xmldsig-more#rsa-sha256",
            "http://www.w3.org/2001/04/xmlenc#sha256",
            EXCLUSIVE_C14N,
            &[
                ENVELOPED_SIGNATURE_TRANSFORM,
                "http://www.w3.org/TR/2001/REC-xml-c14n-20010315",
            ],
        ));
        assert!(!ok);
        assert!(
            errors
                .iter()
                .any(|e| e.contains("disallowed canonicalization transform")),
            "{errors:?}"
        );
    }

    #[test]
    fn check_signature_algorithms_requires_enveloped_transform() {
        // eID §9.1: the signature is embedded with the enveloped-signature
        // transform; a Reference without it did not digest our message root.
        let (ok, errors) = algorithm_errors(&signed_info(
            "http://www.w3.org/2001/04/xmldsig-more#rsa-sha256",
            "http://www.w3.org/2001/04/xmlenc#sha256",
            EXCLUSIVE_C14N,
            &[EXCLUSIVE_C14N],
        ));
        assert!(!ok);
        assert!(
            errors
                .iter()
                .any(|e| e.contains("enveloped-signature transform")),
            "{errors:?}"
        );
    }

    #[test]
    fn check_signature_algorithms_accepts_sha256_and_rejects_sha1() {
        let ok_xml = signed_info(
            "http://www.w3.org/2001/04/xmldsig-more#rsa-sha256",
            "http://www.w3.org/2001/04/xmlenc#sha256",
            EXCLUSIVE_C14N,
            &[ENVELOPED_SIGNATURE_TRANSFORM, EXCLUSIVE_C14N],
        );
        let ok_xml = ok_xml.as_str();
        let doc = crate::saml::xml_parser::parse(ok_xml).unwrap();
        let sig = doc.document_element();
        let mut errors = Vec::new();
        assert!(
            check_signature_algorithms(&doc, sig, &mut errors),
            "{errors:?}"
        );

        // rsa-sha1 SignatureMethod + sha1 DigestMethod must both be rejected.
        let sha1_xml = signed_info(
            "http://www.w3.org/2000/09/xmldsig#rsa-sha1",
            "http://www.w3.org/2000/09/xmldsig#sha1",
            EXCLUSIVE_C14N,
            &[ENVELOPED_SIGNATURE_TRANSFORM, EXCLUSIVE_C14N],
        );
        let sha1_xml = sha1_xml.as_str();
        let doc = crate::saml::xml_parser::parse(sha1_xml).unwrap();
        let sig = doc.document_element();
        let mut errors = Vec::new();
        assert!(!check_signature_algorithms(&doc, sig, &mut errors));
        assert!(errors.iter().any(|e| e.contains("SignatureMethod")));
        assert!(errors.iter().any(|e| e.contains("DigestMethod")));
    }

    #[test]
    fn signature_covers_root_binds_reference_to_root() {
        // Reference "#_root" with a matching root ID, and empty URI, both pass.
        for uri in ["#_root", ""] {
            let xml = format!(
                r##"<Root xmlns="x" ID="_root"><Signature xmlns="{NS_DSIG}"><SignedInfo><Reference URI="{uri}"/></SignedInfo></Signature></Root>"##
            );
            let doc = crate::saml::xml_parser::parse(&xml).unwrap();
            let root = doc.document_element();
            let sig =
                crate::saml::xml_parser::find_child(&doc, root, NS_DSIG, "Signature").unwrap();
            let mut errors = Vec::new();
            assert!(
                signature_covers_root(&doc, root, sig, &mut errors),
                "{errors:?}"
            );
        }

        // A reference to a sibling/other id must be rejected (XSW wrapping).
        let xml = format!(
            r##"<Root xmlns="x" ID="_root"><Signature xmlns="{NS_DSIG}"><SignedInfo><Reference URI="#_sibling"/></SignedInfo></Signature></Root>"##
        );
        let doc = crate::saml::xml_parser::parse(&xml).unwrap();
        let root = doc.document_element();
        let sig = crate::saml::xml_parser::find_child(&doc, root, NS_DSIG, "Signature").unwrap();
        let mut errors = Vec::new();
        assert!(!signature_covers_root(&doc, root, sig, &mut errors));
        assert!(errors.iter().any(|e| e.contains("wrapping")));
    }

    #[test]
    fn nested_signature_before_enveloping_is_rejected() {
        // A <Signature> nested in an earlier subtree precedes the enveloping
        // (direct-child) signature in document order. The backend would verify
        // that nested one, so verify_xml_signature must reject the document as a
        // possible signature-wrapping attack before any crypto runs.
        let xml = format!(
            r##"<Root xmlns="urn:x" ID="_root"><Wrapper><Signature xmlns="{NS_DSIG}"><SignedInfo><Reference URI="#_inner"/></SignedInfo><KeyInfo><KeyName>x</KeyName></KeyInfo></Signature></Wrapper><Signature xmlns="{NS_DSIG}"><SignedInfo><Reference URI="#_root"/></SignedInfo><KeyInfo><KeyName>x</KeyName></KeyInfo></Signature></Root>"##
        );
        let result = verify_xml_signature(&xml, &[key_pair(TEST_PEM)]);
        assert!(!result.is_valid());
        assert!(
            result.errors.iter().any(|e| e.contains("wrapping")),
            "{:?}",
            result.errors
        );
    }

    #[test]
    fn find_matching_key_reports_unknown_key_name() {
        let kp = key_pair(TEST_PEM);
        let xml = r#"<Signature xmlns="http://www.w3.org/2000/09/xmldsig#"><KeyInfo><KeyName>deadbeef</KeyName></KeyInfo></Signature>"#;
        let doc = crate::saml::xml_parser::parse(xml).unwrap();
        let sig = doc.document_element();
        let mut errors = Vec::new();
        assert!(find_matching_key(&doc, sig, std::slice::from_ref(&kp), &mut errors).is_none());
        assert!(errors[0].contains("Unknown KeyName"));
    }
}
