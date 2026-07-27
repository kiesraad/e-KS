//! SOAP-wrapped ArtifactResponse validation (eID §7.6.1).

use super::helpers::{
    check_instant_within, check_issuer, check_status_success, check_version, child_element,
};
use crate::{
    keys::KeyPair,
    saml::{
        constants::*,
        verification::verify_xml_signature,
        xml_parser::{Document, NodeId, children_by_tag},
    },
};
use chrono::{Duration, Utc};
use tracing::debug;

/// Validate the ArtifactResponse element `art_node` within the already-parsed
/// document `doc` (eID §7.6.1) and return the inner Response node.
///
/// Checks (all §7.6.1): `@Version`, signature, Issuer, InResponseTo, status code
/// and `@IssueInstant` staleness.
/// eID §9.2: the signature verification key MUST come from verified RD metadata.
/// `expected_issuer` is the pinned RD EntityID; `None` skips the Issuer binding
/// (tests). The whole signed document is parsed exactly once and navigated here,
/// so no namespace-incomplete subtree is ever re-parsed.
pub fn validate_artifact_response_at(
    doc: &Document,
    art_node: NodeId,
    trusted_keys: &[KeyPair],
    expected_in_response_to: &str,
    expected_issuer: Option<&str>,
    errors: &mut Vec<String>,
) -> Option<NodeId> {
    let root_name = doc.local_name(art_node).unwrap_or_default();
    if root_name != "ArtifactResponse" {
        errors.push(format!("Expected ArtifactResponse, got {root_name}"));
    }

    check_version(doc, art_node, "ArtifactResponse", errors);
    check_artifact_response_signature(doc, art_node, trusted_keys, errors);
    // eID §7.6.1: the Issuer MUST be the RD EntityID. The signature is already
    // verified against an RD signing cert from metadata, but binding the Issuer
    // element mirrors the Response/Assertion checks and rejects a signed-by-RD
    // envelope whose Issuer nonetheless names a different entity.
    check_issuer(doc, art_node, expected_issuer, "ArtifactResponse", errors);
    check_in_response_to(doc, art_node, expected_in_response_to, errors);
    // eID §7.6.1: require Success; the second-level StatusCode and StatusMessage
    // carry the actual reason and are composed into the error (§7.8).
    let status_code = check_status_success(doc, art_node, "ArtifactResponse", errors);

    // Bound how stale the ArtifactResponse envelope may be (it carries no Conditions).
    check_instant_within(
        doc.get_attribute(art_node, "IssueInstant"),
        Utc::now(),
        Duration::seconds(CLOCK_SKEW_SECONDS),
        Duration::seconds(MESSAGE_FRESHNESS_SECONDS),
        "ArtifactResponse @IssueInstant",
        errors,
    );

    // Extract the inner Response from the PARSED tree (comment-safe, anti-XSW;
    // see `child_element`).
    let response = child_element(doc, art_node, NS_SAMLP, "Response");
    debug!(
        "[validate] Extracted inner Response: present={}",
        response.is_some()
    );
    // eID §7.6.1 (and SAML-bindings §3.6.6, quoted in §7.6.1): `Response` is
    // conditional, and a `Success` status does NOT guarantee it is present (the
    // requester may be unauthorized or the artifact already spent). Report the
    // missing Response on the success path regardless of other errors, so the
    // caller never treats "signature fine, status Success" as a usable response.
    if response.is_none() && status_code.as_deref() == Some(STATUS_SUCCESS) {
        errors.push(
            "ArtifactResponse reports Success but carries no Response (artifact expired, \
             already resolved, or requester not authorized)"
                .to_string(),
        );
    }
    // More than one `Response` child is ambiguous: SAML-bindings §3.6.6 allows at
    // most one, and we consume only the first, so reject rather than pick.
    let response_count = children_by_tag(doc, art_node, NS_SAMLP, "Response").len();
    if response_count > 1 {
        errors.push(format!(
            "ArtifactResponse carries {response_count} Response elements (at most one is allowed)"
        ));
    }

    debug!(
        "[validate] ArtifactResponse done: valid={}, errors={}",
        errors.is_empty(),
        errors.len()
    );
    response
}

fn check_artifact_response_signature(
    doc: &Document,
    art_node: NodeId,
    trusted_keys: &[KeyPair],
    errors: &mut Vec<String>,
) {
    debug!("[validate] Verifying ArtifactResponse XML signature");
    // The ArtifactResponse element carries its own namespace declarations, so its
    // source bytes are self-contained input for canonicalization/verification.
    let Some(xml) = doc.node_source(art_node) else {
        errors.push("ArtifactResponse sig: could not read element source".to_string());
        return;
    };
    let sig_result = verify_xml_signature(xml, trusted_keys);
    if !sig_result.is_valid() {
        errors.extend(
            sig_result
                .errors
                .iter()
                .map(|e| format!("ArtifactResponse sig: {e}")),
        );
    } else {
        debug!("[validate] ArtifactResponse signature OK");
    }
}

fn check_in_response_to(doc: &Document, root: NodeId, expected: &str, errors: &mut Vec<String>) {
    let in_response_to = doc.get_attribute(root, "InResponseTo").unwrap_or("");
    debug!("[validate] ArtifactResponse InResponseTo='{in_response_to}' (expected='{expected}')");
    if !expected.is_empty() && in_response_to != expected {
        errors.push(format!(
            "ArtifactResponse InResponseTo mismatch: expected {expected}, got {in_response_to}"
        ));
    }
}
