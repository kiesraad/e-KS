//! Shared XML/SOAP helpers for the SAML validators.

use crate::saml::{
    constants::{NS_SAML, NS_SAMLP, STATUS_SUCCESS},
    xml_parser::{Document, NodeId, find_child, find_descendant, inner_text},
};
use chrono::{DateTime, Duration, Utc};

/// eID §7.6.1/§7.6.2/§7.6.3 (and SAML core): `@Version` MUST be exactly `2.0`.
/// A missing or differing version fails closed rather than being ignored.
pub(super) fn check_version(doc: &Document, node: NodeId, label: &str, errors: &mut Vec<String>) {
    match doc.get_attribute(node, "Version") {
        Some("2.0") => {}
        Some(v) => errors.push(format!(
            "{label} has unsupported @Version {v:?} (MUST be 2.0)"
        )),
        None => errors.push(format!("{label} is missing the required @Version")),
    }
}

/// eID §7.6.1/§7.6.2/§7.6.3.5 rule 1: the `<saml:Issuer>` of `node` MUST be the
/// pinned RD EntityID, so an RD-signed envelope naming a different entity is
/// rejected. `None` skips the check (tests).
pub(super) fn check_issuer(
    doc: &Document,
    node: NodeId,
    expected_issuer: Option<&str>,
    label: &str,
    errors: &mut Vec<String>,
) {
    let Some(expected) = expected_issuer else {
        return;
    };
    match find_child(doc, node, NS_SAML, "Issuer").map(|n| inner_text(doc, n)) {
        Some(ref i) if i.trim() == expected => {}
        Some(i) => errors.push(format!(
            "{label} Issuer mismatch: expected {expected}, got {}",
            i.trim()
        )),
        None => errors.push(format!("{label} has no Issuer")),
    }
}

/// eID §7.6.1/§7.6.2: require a `Success` StatusCode on `node`, composing the
/// second-level StatusCode and StatusMessage (§7.8) into the error so the actual
/// reason is visible in logs. Returns the top-level status code for callers that
/// branch on it.
pub(super) fn check_status_success(
    doc: &Document,
    node: NodeId,
    label: &str,
    errors: &mut Vec<String>,
) -> Option<String> {
    let status_code = find_status_code(doc, node);
    if status_code.as_deref() != Some(STATUS_SUCCESS) {
        let second = find_nested_status_code(doc, node);
        let message = find_samlp_text(doc, node, "StatusMessage");
        errors.push(format!(
            "{label} status: {}{}{}",
            status_code.as_deref().unwrap_or("unknown"),
            second.map(|s| format!(" ({s})")).unwrap_or_default(),
            message.map(|m| format!(" - {m}")).unwrap_or_default()
        ));
    }
    status_code
}

fn find_status_code(doc: &Document, root: NodeId) -> Option<String> {
    find_descendant(doc, root, NS_SAMLP, "StatusCode")
        .and_then(|n| doc.get_attribute(n, "Value"))
        .map(String::from)
}

fn find_nested_status_code(doc: &Document, root: NodeId) -> Option<String> {
    let sc = find_descendant(doc, root, NS_SAMLP, "StatusCode")?;
    find_child(doc, sc, NS_SAMLP, "StatusCode")
        .and_then(|n| doc.get_attribute(n, "Value"))
        .map(String::from)
}

/// Find a `samlp:`-namespaced descendant element's text (e.g. `StatusMessage`).
fn find_samlp_text(doc: &Document, root: NodeId, local_name: &str) -> Option<String> {
    find_descendant(doc, root, NS_SAMLP, local_name).map(|n| inner_text(doc, n))
}

/// Bound an `@IssueInstant`/`@AuthnInstant` on both sides: reject a value older
/// than `max_age` (plus skew) and one further in the future than skew allows.
///
/// eID §7.6.1/§7.6.2/§7.6.3 give `@IssueInstant` cardinality 1 on every message
/// we receive and §7.6.3 the same for `@AuthnInstant`, so an absent or
/// unparseable value fails closed rather than skipping the freshness bound.
///
/// The spec sets no explicit freshness window (it bounds the Assertion only via
/// `Conditions`), so `max_age` is our own ceiling: the envelope types carry no
/// `Conditions` at all, which would otherwise leave them unbounded in time.
/// The future bound matters for the same reason: without it a far-future
/// `@IssueInstant` would never expire.
pub(super) fn check_instant_within(
    val: Option<&str>,
    now: DateTime<Utc>,
    skew: Duration,
    max_age: Duration,
    label: &str,
    errors: &mut Vec<String>,
) {
    let Some(s) = val else {
        errors.push(format!("{label} is missing (required, cardinality 1)"));
        return;
    };
    match s.parse::<DateTime<Utc>>() {
        Ok(t) if t + max_age + skew < now => {
            errors.push(format!("{label} is stale: issued at {s}"));
        }
        Ok(t) if t - skew > now => {
            errors.push(format!("{label} is in the future: issued at {s}"));
        }
        Ok(_) => {}
        Err(_) => errors.push(format!("{label} has an invalid timestamp: {s}")),
    }
}

/// Parse a mandatory timestamp attribute, failing closed: a missing or
/// unparseable value records an error and yields `None`. On success the parsed
/// instant is returned together with the raw string (for error messages).
fn parse_required_instant<'a>(
    val: Option<&'a str>,
    attr: &str,
    label: &str,
    errors: &mut Vec<String>,
) -> Option<(DateTime<Utc>, &'a str)> {
    let Some(s) = val else {
        errors.push(format!("{label} is missing the required {attr}"));
        return None;
    };
    match s.parse::<DateTime<Utc>>() {
        Ok(t) => Some((t, s)),
        Err(_) => {
            errors.push(format!("{label} has an invalid {attr} timestamp: {s}"));
            None
        }
    }
}

/// eID §7.6.3 (cardinality 1): `Conditions/@NotBefore` is mandatory (stricter
/// than SAML core, where it is optional). Reject a missing, unparseable, or
/// not-yet-valid value.
pub(super) fn check_not_before(
    val: Option<&str>,
    now: DateTime<Utc>,
    skew: Duration,
    label: &str,
    errors: &mut Vec<String>,
) {
    if let Some((t, s)) = parse_required_instant(val, "NotBefore", label, errors)
        && t - skew > now
    {
        errors.push(format!("{label} not yet valid: NotBefore {s}"));
    }
}

/// eID §7.6.3 (cardinality 1) / §9.5: @NotOnOrAfter is mandatory, so a missing
/// or unparseable value fails closed rather than silently skipping the
/// freshness check (an assertion with no/garbage expiry must not be accepted).
pub(super) fn check_not_on_or_after(
    val: Option<&str>,
    now: DateTime<Utc>,
    skew: Duration,
    label: &str,
    errors: &mut Vec<String>,
) {
    if let Some((t, s)) = parse_required_instant(val, "NotOnOrAfter", label, errors)
        && t + skew < now
    {
        errors.push(format!("{label} expired: {s}"));
    }
}

/// Find a direct child element `(ns, local_name)` as a node in the parsed tree.
///
/// SECURITY (XML Signature Wrapping): exclusive-c14n and roxmltree both exclude
/// comments, so a raw string scan could slice a forged element out of a comment
/// interior that the signature digest never covered. Navigating the single
/// parsed tree (instead of re-scanning bytes) reads exactly the element the
/// signed, comment-excluded view sees.
pub(super) fn child_element(
    doc: &Document,
    parent: NodeId,
    ns: &str,
    local_name: &str,
) -> Option<NodeId> {
    find_child(doc, parent, ns, local_name)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::saml::xml_parser::parse;

    fn root_of(doc: &Document) -> NodeId {
        doc.document_element()
    }

    // -- child_element (parser-based, anti-XSW) --

    #[test]
    fn child_element_returns_direct_child() {
        let xml = format!(
            r#"<samlp:ArtifactResponse xmlns:samlp="{NS_SAMLP}"><samlp:Response ID="_r1">inner</samlp:Response></samlp:ArtifactResponse>"#
        );
        let doc = parse(&xml).unwrap();
        let child = child_element(&doc, root_of(&doc), NS_SAMLP, "Response").unwrap();
        assert_eq!(doc.get_attribute(child, "ID"), Some("_r1"));
    }

    #[test]
    fn child_element_none_when_missing() {
        let xml = format!(
            r#"<samlp:ArtifactResponse xmlns:samlp="{NS_SAMLP}">nothing</samlp:ArtifactResponse>"#
        );
        let doc = parse(&xml).unwrap();
        assert!(child_element(&doc, root_of(&doc), NS_SAMLP, "Response").is_none());
    }

    #[test]
    fn child_element_ignores_element_hidden_in_comment() {
        // SECURITY (XSW): a forged <Response> hidden in a comment must NOT be
        // found; the parser never materializes comment content, so the node
        // lookup returns the genuine direct child only.
        let xml = format!(
            r#"<samlp:ArtifactResponse xmlns:samlp="{NS_SAMLP}"><!--<samlp:Response ID="_forged">EVIL</samlp:Response>--><samlp:Response ID="_genuine">GOOD</samlp:Response></samlp:ArtifactResponse>"#
        );
        let doc = parse(&xml).unwrap();
        let child = child_element(&doc, root_of(&doc), NS_SAMLP, "Response").unwrap();
        assert_eq!(doc.get_attribute(child, "ID"), Some("_genuine"));
        assert_eq!(inner_text(&doc, child), "GOOD");
    }

    // -- find_status_code --

    #[test]
    fn find_status_code_success() {
        let xml = format!(
            r#"<Response xmlns="{NS_SAMLP}"><Status><StatusCode Value="{STATUS_SUCCESS}"/></Status></Response>"#
        );
        let doc = parse(&xml).unwrap();
        assert_eq!(
            find_status_code(&doc, root_of(&doc)).unwrap(),
            STATUS_SUCCESS
        );
    }

    #[test]
    fn find_status_code_missing() {
        let xml = format!(r#"<Response xmlns="{NS_SAMLP}"/>"#);
        let doc = parse(&xml).unwrap();
        assert!(find_status_code(&doc, root_of(&doc)).is_none());
    }

    // -- find_nested_status_code --

    #[test]
    fn find_nested_status_code_extracts_second_level() {
        let xml = format!(
            r#"<Response xmlns="{NS_SAMLP}"><Status><StatusCode Value="urn:oasis:names:tc:SAML:2.0:status:Responder"><StatusCode Value="urn:oasis:names:tc:SAML:2.0:status:AuthnFailed"/></StatusCode></Status></Response>"#
        );
        let doc = parse(&xml).unwrap();
        let nested = find_nested_status_code(&doc, root_of(&doc)).unwrap();
        assert!(nested.contains("AuthnFailed"));
    }

    // -- check_not_on_or_after --

    #[test]
    fn check_not_on_or_after_valid() {
        let mut errors = Vec::new();
        let future = (Utc::now() + Duration::hours(1))
            .format("%Y-%m-%dT%H:%M:%SZ")
            .to_string();
        check_not_on_or_after(
            Some(&future),
            Utc::now(),
            Duration::seconds(30),
            "Test",
            &mut errors,
        );
        assert!(errors.is_empty());
    }

    #[test]
    fn check_not_on_or_after_expired() {
        let mut errors = Vec::new();
        let past = (Utc::now() - Duration::hours(1))
            .format("%Y-%m-%dT%H:%M:%SZ")
            .to_string();
        check_not_on_or_after(
            Some(&past),
            Utc::now(),
            Duration::seconds(30),
            "Test",
            &mut errors,
        );
        assert_eq!(errors.len(), 1);
        assert!(errors[0].contains("expired"));
    }

    #[test]
    fn check_instant_within_bounds_both_directions_and_requires_presence() {
        let now = Utc::now();
        let skew = Duration::seconds(30);
        let max_age = Duration::seconds(300);
        let at = |offset: Duration| (now + offset).format("%Y-%m-%dT%H:%M:%SZ").to_string();

        // A recent instant is fresh.
        let mut errors = Vec::new();
        check_instant_within(
            Some(&at(-Duration::seconds(10))),
            now,
            skew,
            max_age,
            "Msg",
            &mut errors,
        );
        assert!(errors.is_empty(), "fresh instant: {errors:?}");

        // eID cardinality 1: an absent instant fails closed.
        let mut errors = Vec::new();
        check_instant_within(None, now, skew, max_age, "Msg", &mut errors);
        assert!(
            errors.iter().any(|e| e.contains("is missing")),
            "{errors:?}"
        );

        // Older than max_age + skew is stale.
        let mut errors = Vec::new();
        check_instant_within(
            Some(&at(-Duration::seconds(400))),
            now,
            skew,
            max_age,
            "Msg",
            &mut errors,
        );
        assert!(errors.iter().any(|e| e.contains("stale")), "{errors:?}");

        // Further in the future than skew allows is rejected too: without this a
        // far-future instant would never age out.
        let mut errors = Vec::new();
        check_instant_within(
            Some(&at(Duration::hours(1))),
            now,
            skew,
            max_age,
            "Msg",
            &mut errors,
        );
        assert!(
            errors.iter().any(|e| e.contains("in the future")),
            "{errors:?}"
        );

        // Inside the skew allowance a slightly-future instant is still accepted.
        let mut errors = Vec::new();
        check_instant_within(
            Some(&at(Duration::seconds(5))),
            now,
            skew,
            max_age,
            "Msg",
            &mut errors,
        );
        assert!(errors.is_empty(), "within skew: {errors:?}");

        // An unparseable instant fails closed.
        let mut errors = Vec::new();
        check_instant_within(Some("garbage"), now, skew, max_age, "Msg", &mut errors);
        assert!(errors.iter().any(|e| e.contains("invalid timestamp")));
    }

    #[test]
    fn check_not_before_requires_presence_and_rejects_future() {
        let now = Utc::now();
        let skew = Duration::seconds(30);
        let at = |offset: Duration| (now + offset).format("%Y-%m-%dT%H:%M:%SZ").to_string();

        // eID §7.6.3 makes Conditions/@NotBefore mandatory.
        let mut errors = Vec::new();
        check_not_before(None, now, skew, "Test", &mut errors);
        assert!(
            errors.iter().any(|e| e.contains("missing the required")),
            "{errors:?}"
        );

        let mut errors = Vec::new();
        check_not_before(
            Some(&at(Duration::hours(1))),
            now,
            skew,
            "Test",
            &mut errors,
        );
        assert!(
            errors.iter().any(|e| e.contains("not yet valid")),
            "{errors:?}"
        );

        let mut errors = Vec::new();
        check_not_before(Some("nonsense"), now, skew, "Test", &mut errors);
        assert!(errors.iter().any(|e| e.contains("invalid NotBefore")));

        let mut errors = Vec::new();
        check_not_before(
            Some(&at(-Duration::minutes(5))),
            now,
            skew,
            "Test",
            &mut errors,
        );
        assert!(errors.is_empty(), "past NotBefore: {errors:?}");
    }

    #[test]
    fn check_not_on_or_after_missing_or_malformed_fails_closed() {
        // Absent @NotOnOrAfter is a mandatory-element violation, not "no check".
        let mut errors = Vec::new();
        check_not_on_or_after(None, Utc::now(), Duration::seconds(30), "Test", &mut errors);
        assert!(errors.iter().any(|e| e.contains("missing the required")));

        // An unparseable timestamp must also be rejected, not silently accepted.
        let mut errors = Vec::new();
        check_not_on_or_after(
            Some("not-a-timestamp"),
            Utc::now(),
            Duration::seconds(30),
            "Test",
            &mut errors,
        );
        assert!(errors.iter().any(|e| e.contains("invalid NotOnOrAfter")));
    }
}
