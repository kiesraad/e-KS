//! Inner Response validation (eID §7.6.2).

use super::helpers::{
    check_instant_within, check_issuer, check_status_success, check_version, child_element,
};
use crate::saml::{
    constants::*,
    xml_parser::{Document, NodeId, children_by_tag, find_descendant},
};
use chrono::{Duration, Utc};
use tracing::debug;

/// Validate the Response element `response` within the already-parsed document
/// `doc` (eID §7.6.2) and return the inner Assertion node.
///
/// eID §7.6.2: Response MUST contain Status with StatusCode; if not Success, a
/// second-level StatusCode SHOULD be present (§7.8). Assertion MUST be present on
/// Success; EncryptedAssertion MUST NOT be included. `@Destination` MUST match the
/// recipient ACS and Issuer MUST be the RD EntityID (checked when supplied).
pub fn validate_response_at(
    doc: &Document,
    response: NodeId,
    expected_destination: Option<&str>,
    expected_issuer: Option<&str>,
    errors: &mut Vec<String>,
) -> Option<NodeId> {
    check_version(doc, response, "Response", errors);

    // Bound how stale the Response envelope may be (it carries no Conditions).
    check_instant_within(
        doc.get_attribute(response, "IssueInstant"),
        Utc::now(),
        Duration::seconds(CLOCK_SKEW_SECONDS),
        Duration::seconds(MESSAGE_FRESHNESS_SECONDS),
        "Response @IssueInstant",
        errors,
    );

    let status_code = check_status_success(doc, response, "Response", errors);
    debug!(
        "[validate] Response status_code={:?}",
        status_code.as_deref()
    );

    // eID §7.6.2: EncryptedAssertion MUST NOT be included (the Assertion travels
    // in plaintext inside the RD-signed ArtifactResponse; only the SubjectIDs are
    // encrypted, per §7.6.3.4).
    if find_descendant(doc, response, NS_SAML, "EncryptedAssertion").is_some() {
        errors
            .push("Response contains an EncryptedAssertion, which eID §7.6.2 forbids".to_string());
    }

    // eID §7.6.2: @Destination MUST match the recipient ACS the artifact was
    // delivered to. Mirrors the assertion-level Recipient binding (§7.6.3.5 r2).
    if let Some(expected) = expected_destination {
        let destination = doc.get_attribute(response, "Destination").unwrap_or("");
        debug!("[validate] Response Destination='{destination}' (expected='{expected}')");
        if destination != expected {
            errors.push(format!(
                "Response Destination mismatch: expected {expected}, got {destination}"
            ));
        }
    }

    // eID §7.6.2: Issuer MUST be the RD EntityID. Mirrors the assertion-level
    // Issuer binding (§7.6.3.5 r1).
    check_issuer(doc, response, expected_issuer, "Response", errors);

    // Extract the Assertion from the PARSED tree (comment-safe, anti-XSW; see
    // `child_element`).
    let assertion = child_element(doc, response, NS_SAML, "Assertion");
    debug!(
        "[validate] Extracted Assertion: present={}",
        assertion.is_some()
    );

    // eID §7.6.2 (Assertion cardinality 0..1, conditional): MUST be present when
    // the status is Success and MUST NOT be included otherwise.
    let is_success = status_code.as_deref() == Some(STATUS_SUCCESS);
    if assertion.is_none() && is_success {
        errors.push("No Assertion found in successful Response".to_string());
    }
    if assertion.is_some() && !is_success {
        errors.push(
            "Response carries an Assertion without a Success status, which eID §7.6.2 forbids"
                .to_string(),
        );
    }
    // More than one Assertion is ambiguous (cardinality 0..1) and we consume only
    // the first, so reject rather than silently pick one.
    let assertion_count = children_by_tag(doc, response, NS_SAML, "Assertion").len();
    if assertion_count > 1 {
        errors.push(format!(
            "Response carries {assertion_count} Assertion elements (at most one is allowed)"
        ));
    }

    debug!(
        "[validate] Response done: valid={}, errors={}",
        errors.is_empty(),
        errors.len()
    );
    assertion
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::saml::xml_parser::parse;

    /// A SAML timestamp `offset` from now, for the mandatory `@IssueInstant`.
    fn ts(offset: chrono::Duration) -> String {
        (Utc::now() + offset)
            .format("%Y-%m-%dT%H:%M:%SZ")
            .to_string()
    }

    /// Parse `xml` and run [`validate_response_at`] over its root, returning
    /// `(valid, errors, assertion_source)`.
    fn run(
        xml: &str,
        dest: Option<&str>,
        issuer: Option<&str>,
    ) -> (bool, Vec<String>, Option<String>) {
        let doc = parse(xml).expect("test XML parses");
        let root = doc.document_element();
        let mut errors = Vec::new();
        let assertion = validate_response_at(&doc, root, dest, issuer, &mut errors);
        let assertion_xml = assertion.and_then(|n| doc.node_source(n).map(str::to_string));
        (errors.is_empty(), errors, assertion_xml)
    }

    #[test]
    fn validate_response_success_with_assertion() {
        let xml = format!(
            r#"<samlp:Response xmlns:samlp="{NS_SAMLP}" Version="2.0" IssueInstant="{now}"><samlp:Status><samlp:StatusCode Value="{STATUS_SUCCESS}"/></samlp:Status><saml:Assertion xmlns:saml="{NS_SAML}" ID="_a1">data</saml:Assertion></samlp:Response>"#,
            now = ts(chrono::Duration::zero())
        );
        let (valid, errors, assertion_xml) = run(&xml, None, None);
        assert!(valid, "Errors: {errors:?}");
        assert!(assertion_xml.is_some());
    }

    #[test]
    fn validate_response_error_status() {
        let xml = format!(
            r#"<samlp:Response xmlns:samlp="{NS_SAMLP}"><samlp:Status><samlp:StatusCode Value="urn:oasis:names:tc:SAML:2.0:status:Responder"/></samlp:Status></samlp:Response>"#
        );
        let (valid, errors, _) = run(&xml, None, None);
        assert!(!valid);
        assert!(errors.iter().any(|e| e.contains("Responder")));
    }

    #[test]
    fn validate_response_missing_assertion_on_success() {
        let xml = format!(
            r#"<samlp:Response xmlns:samlp="{NS_SAMLP}"><samlp:Status><samlp:StatusCode Value="{STATUS_SUCCESS}"/></samlp:Status></samlp:Response>"#
        );
        let (valid, errors, _) = run(&xml, None, None);
        assert!(!valid);
        assert!(errors.iter().any(|e| e.contains("No Assertion")));
    }

    #[test]
    fn validate_response_rejects_missing_version() {
        // eID §7.6.2: @Version is mandatory and MUST be 2.0; its absence fails closed.
        let xml = format!(
            r#"<samlp:Response xmlns:samlp="{NS_SAMLP}"><samlp:Status><samlp:StatusCode Value="{STATUS_SUCCESS}"/></samlp:Status><saml:Assertion xmlns:saml="{NS_SAML}" ID="_a1">data</saml:Assertion></samlp:Response>"#
        );
        let (valid, errors, _) = run(&xml, None, None);
        assert!(!valid);
        assert!(
            errors
                .iter()
                .any(|e| e.contains("missing the required @Version")),
            "{errors:?}"
        );
    }

    #[test]
    fn validate_response_rejects_wrong_version() {
        let xml = format!(
            r#"<samlp:Response xmlns:samlp="{NS_SAMLP}" Version="1.1"><samlp:Status><samlp:StatusCode Value="{STATUS_SUCCESS}"/></samlp:Status><saml:Assertion xmlns:saml="{NS_SAML}" ID="_a1">data</saml:Assertion></samlp:Response>"#
        );
        let (valid, errors, _) = run(&xml, None, None);
        assert!(!valid);
        assert!(
            errors.iter().any(|e| e.contains("unsupported @Version")),
            "{errors:?}"
        );
    }

    #[test]
    fn validate_response_rejects_encrypted_assertion() {
        // eID §7.6.2: EncryptedAssertion MUST NOT be present.
        let xml = format!(
            r#"<samlp:Response xmlns:samlp="{NS_SAMLP}"><samlp:Status><samlp:StatusCode Value="{STATUS_SUCCESS}"/></samlp:Status><saml:EncryptedAssertion xmlns:saml="{NS_SAML}"><xenc:EncryptedData xmlns:xenc="http://www.w3.org/2001/04/xmlenc#"/></saml:EncryptedAssertion></samlp:Response>"#
        );
        let (valid, errors, _) = run(&xml, None, None);
        assert!(!valid);
        assert!(errors.iter().any(|e| e.contains("EncryptedAssertion")));
    }

    #[test]
    fn validate_response_checks_destination_and_issuer() {
        let xml = format!(
            r#"<samlp:Response xmlns:samlp="{NS_SAMLP}" Version="2.0" IssueInstant="{now}" Destination="https://dv.test/acs"><saml:Issuer xmlns:saml="{NS_SAML}">urn:rd</saml:Issuer><samlp:Status><samlp:StatusCode Value="{STATUS_SUCCESS}"/></samlp:Status><saml:Assertion xmlns:saml="{NS_SAML}" ID="_a1">x</saml:Assertion></samlp:Response>"#,
            now = ts(chrono::Duration::zero())
        );

        // Matching destination + issuer: accepted.
        let (valid, errors, _) = run(&xml, Some("https://dv.test/acs"), Some("urn:rd"));
        assert!(valid, "errors: {errors:?}");

        // Wrong destination: rejected.
        let (_, errors, _) = run(&xml, Some("https://attacker.test/acs"), Some("urn:rd"));
        assert!(errors.iter().any(|e| e.contains("Destination mismatch")));

        // Wrong issuer: rejected.
        let (_, errors, _) = run(&xml, Some("https://dv.test/acs"), Some("urn:someone-else"));
        assert!(errors.iter().any(|e| e.contains("Issuer mismatch")));
    }

    #[test]
    fn validate_response_rejects_assertion_without_success_status() {
        // eID §7.6.2: the Assertion MUST NOT be included unless the status is
        // Success, so a failure status shipping an assertion is a protocol
        // violation, not something to extract claims from.
        let xml = format!(
            r#"<samlp:Response xmlns:samlp="{NS_SAMLP}" Version="2.0" IssueInstant="{now}"><samlp:Status><samlp:StatusCode Value="urn:oasis:names:tc:SAML:2.0:status:Responder"/></samlp:Status><saml:Assertion xmlns:saml="{NS_SAML}" ID="_a1">data</saml:Assertion></samlp:Response>"#,
            now = ts(chrono::Duration::zero())
        );
        let (valid, errors, _) = run(&xml, None, None);
        assert!(!valid);
        assert!(
            errors
                .iter()
                .any(|e| e.contains("Assertion without a Success status")),
            "{errors:?}"
        );
    }

    #[test]
    fn validate_response_rejects_multiple_assertions() {
        // Cardinality 0..1: two Assertions are ambiguous, and we consume only the
        // first, so an attacker must not be able to append a second one.
        let xml = format!(
            r#"<samlp:Response xmlns:samlp="{NS_SAMLP}" Version="2.0" IssueInstant="{now}"><samlp:Status><samlp:StatusCode Value="{STATUS_SUCCESS}"/></samlp:Status><saml:Assertion xmlns:saml="{NS_SAML}" ID="_a1">first</saml:Assertion><saml:Assertion xmlns:saml="{NS_SAML}" ID="_a2">second</saml:Assertion></samlp:Response>"#,
            now = ts(chrono::Duration::zero())
        );
        let (valid, errors, _) = run(&xml, None, None);
        assert!(!valid);
        assert!(
            errors.iter().any(|e| e.contains("2 Assertion elements")),
            "{errors:?}"
        );
    }

    #[test]
    fn validate_response_requires_issue_instant() {
        // eID §7.6.2 (cardinality 1): @IssueInstant is mandatory. The Response
        // carries no Conditions, so without it nothing bounds the message in time.
        let xml = format!(
            r#"<samlp:Response xmlns:samlp="{NS_SAMLP}" Version="2.0"><samlp:Status><samlp:StatusCode Value="{STATUS_SUCCESS}"/></samlp:Status><saml:Assertion xmlns:saml="{NS_SAML}" ID="_a1">data</saml:Assertion></samlp:Response>"#
        );
        let (valid, errors, _) = run(&xml, None, None);
        assert!(!valid);
        assert!(
            errors
                .iter()
                .any(|e| e.contains("Response @IssueInstant is missing")),
            "{errors:?}"
        );
    }

    #[test]
    fn validate_response_rejects_stale_and_future_issue_instant() {
        for (offset, expected) in [
            (-chrono::Duration::hours(1), "stale"),
            (chrono::Duration::hours(1), "in the future"),
        ] {
            let xml = format!(
                r#"<samlp:Response xmlns:samlp="{NS_SAMLP}" Version="2.0" IssueInstant="{now}"><samlp:Status><samlp:StatusCode Value="{STATUS_SUCCESS}"/></samlp:Status><saml:Assertion xmlns:saml="{NS_SAML}" ID="_a1">data</saml:Assertion></samlp:Response>"#,
                now = ts(offset)
            );
            let (valid, errors, _) = run(&xml, None, None);
            assert!(!valid, "offset {offset:?} must be rejected");
            assert!(
                errors.iter().any(|e| e.contains(expected)),
                "expected {expected}, got {errors:?}"
            );
        }
    }
}
