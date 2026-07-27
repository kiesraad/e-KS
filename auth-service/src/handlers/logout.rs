use crate::{
    bindings::http_post::autosubmit_post_response,
    saml::{
        constants::{NS_SAML, NS_SAMLP, STATUS_SUCCESS},
        messages::create_logout_request,
        verification::verify_xml_signature,
    },
    state::{AuthServiceState, AuthState},
};
use axum::{
    Form,
    extract::{FromRef, State},
    response::{IntoResponse, Redirect, Response},
};
use axum_extra::extract::CookieJar;
use base64::{Engine, engine::general_purpose::STANDARD as BASE64};
use std::collections::HashMap;
use tracing::{debug, error, info, warn};

/// SP-initiated logout per eID §7.7.1 / §3.1.1.1, mounted by the embedding
/// application on a logout route of its choosing.
///
/// eID §3.1.1.1: only SP-initiated logout is supported. Asks the embedding
/// application to terminate its session (`AuthState::logout_session`) and, if
/// a session was active, builds a signed LogoutRequest (§7.7.1) for the
/// browser to POST to the RD's SLO endpoint. Otherwise redirects to
/// `post_logout_redirect`.
pub async fn handle_logout<S>(state: &S, jar: CookieJar, post_logout_redirect: &str) -> Response
where
    S: AuthState,
    AuthServiceState: FromRef<S>,
{
    let auth_state = AuthServiceState::from_ref(state);
    debug!("[logout] Handler entered, calling AuthState::logout_session");
    // The local session is always torn down and the cookie always cleared; the
    // NameID is only present if one was recorded at login (needed to build the
    // LogoutRequest, eID §7.7.1).
    let (cleared_jar, name_id) = state.logout_session(jar).await;
    let Some(name_id) = name_id else {
        debug!(
            "[logout] No SAML NameID recorded; completed local logout, redirecting to post-logout page"
        );
        return (cleared_jar, Redirect::to(post_logout_redirect)).into_response();
    };

    // SECURITY: never log `name_id` itself; it is a SAML TransientID linkable
    // to a specific authentication session.
    debug!(
        "[logout] Active session terminated; name_id_present=true, name_id_len={}",
        name_id.len()
    );

    let cfg = auth_state.auth_config();
    let dv_keys = auth_state.dv_keys();
    // The local session is already gone; if the RD descriptor is not loaded we
    // cannot build a LogoutRequest (no SLO endpoint), but the user is logged out
    // locally, so complete the logout by redirecting home rather than erroring.
    let Some(rd) = auth_state.rd_metadata() else {
        warn!("[logout] RD metadata not loaded; completed local logout, skipping SAML SLO");
        return (cleared_jar, Redirect::to(post_logout_redirect)).into_response();
    };
    let slo_url = &rd.slo_url;

    debug!(
        "[logout] Building LogoutRequest: entity_id={}, slo_url={}",
        cfg.dv.entity_id, slo_url
    );

    match create_logout_request(
        &name_id,
        &cfg.dv.entity_id,
        slo_url,
        dv_keys.primary_signing(),
    ) {
        Ok(msg) => {
            // eID §7.7.2: record the LogoutRequest ID so the matching
            // LogoutResponse's @InResponseTo can be validated and consumed once in
            // `handle_sls`. Reuses the outstanding-request store used for
            // AuthnRequest IDs (both are one-time IDs this DV issued); no extra
            // `AuthState` method is needed.
            state.register_pending_request(msg.id.clone()).await;
            info!(
                "[logout] LogoutRequest created: {}",
                &msg.id[..20.min(msg.id.len())]
            );
            debug!("[logout] LogoutRequest XML built; returning autosubmit form");
            match autosubmit_post_response(slo_url, &msg.xml, "SAMLRequest") {
                Ok(resp) => (cleared_jar, resp).into_response(),
                Err(e) => {
                    // The local session is already gone, so the user *is* logged
                    // out; complete the logout by redirecting home (like the
                    // missing-metadata branch above) rather than erroring.
                    error!("[logout] Failed to build autosubmit form: {e}; skipping SAML SLO");
                    (cleared_jar, Redirect::to(post_logout_redirect)).into_response()
                }
            }
        }
        Err(e) => {
            error!("[logout] Failed to create LogoutRequest: {e}; skipping SAML SLO");
            (cleared_jar, Redirect::to(post_logout_redirect)).into_response()
        }
    }
}

/// POST /saml/sp/logout: receives the RD's LogoutResponse per eID §7.7.2.
///
/// Enforces the §7.7.2 mandatory checks as a gate on treating the response as a
/// genuine RD confirmation: it MUST be a `samlp:LogoutResponse`, MUST carry a
/// valid RD signature, MUST have an `@InResponseTo` matching a LogoutRequest this
/// DV issued (consumed once, so a replayed response is rejected), and SHOULD
/// report `Success`.
///
/// The user's local session was already terminated by [`handle_logout`] *before*
/// any LogoutResponse arrives, so this endpoint holds no session state. A
/// failed/forged/replayed response is therefore logged and dropped but never
/// strands the user: the browser is always redirected to the post-logout page.
pub async fn handle_sls<S>(
    _: crate::SamlLogoutPath,
    State(state): State<S>,
    State(auth_state): State<AuthServiceState>,
    Form(params): Form<HashMap<String, String>>,
) -> Response
where
    S: AuthState,
    AuthServiceState: FromRef<S>,
{
    debug!("[SLS] Handler entered, form fields: {}", params.len());
    let target = auth_state.auth_config().post_logout_redirect();
    let saml_response = match params.get("SAMLResponse") {
        Some(encoded) => match BASE64.decode(encoded.as_bytes()) {
            Ok(bytes) => {
                debug!("[SLS] SAMLResponse base64 decoded ({} bytes)", bytes.len());
                String::from_utf8_lossy(&bytes).to_string()
            }
            Err(e) => {
                // Like every other malformed-LogoutResponse case: log, drop, and
                // never strand the user (the local logout already happened).
                warn!("[SLS] Invalid SAMLResponse base64: {e}; ignoring");
                return Redirect::to(target).into_response();
            }
        },
        None => {
            // POST without a SAMLResponse field: nothing to verify; accept.
            info!("[SLS] LogoutResponse received without SAMLResponse");
            return Redirect::to(target).into_response();
        }
    };

    info!("[SLS] LogoutResponse received");

    // Without the RD descriptor we cannot verify the signature; the user is
    // already logged out locally, so complete the logout rather than erroring.
    let Some(rd) = auth_state.rd_metadata() else {
        warn!("[SLS] RD metadata not loaded; cannot verify LogoutResponse, redirecting");
        return Redirect::to(target).into_response();
    };

    // eID §7.7.2: the LogoutResponse MUST carry a valid RD signature. A response
    // that does not verify is dropped (the local logout already happened).
    let sig_result = verify_xml_signature(&saml_response, &rd.signing_keys);
    if !sig_result.is_valid() {
        warn!(
            "[SLS] LogoutResponse signature invalid ({:?}); ignoring (local logout already done)",
            sig_result.errors
        );
        return Redirect::to(target).into_response();
    }

    // eID §7.7.2: bind the response to the RD and our SLS endpoint and lift out
    // the correlation fields as owned values, so nothing borrows the parsed
    // document across the async consume below. Any failure is logged and dropped
    // (the local logout already happened in `handle_logout`).
    let cfg = auth_state.auth_config();
    let fields = match extract_logout_response_fields(
        &saml_response,
        rd.entity_id.as_str(),
        cfg.dv.slo_url.as_str(),
    ) {
        Ok(fields) => fields,
        Err(reason) => {
            warn!("[SLS] {reason}; ignoring");
            return Redirect::to(target).into_response();
        }
    };

    // eID §7.7.2: @InResponseTo MUST match a LogoutRequest this DV issued. Consume
    // it once (only after the signature verified, so a forged response cannot burn
    // a pending ID) to reject replays of a captured LogoutResponse.
    if !state.consume_if_pending(fields.in_response_to).await {
        warn!(
            "[SLS] LogoutResponse InResponseTo did not match an outstanding LogoutRequest \
             (unknown, expired, or replayed); ignoring"
        );
        return Redirect::to(target).into_response();
    }

    // eID §7.7.2: report the status. A non-Success status does not change the
    // outcome (the local logout already completed) but is surfaced for diagnostics.
    if fields.status_is_success {
        info!("[SLS] LogoutResponse confirmed by RD (signed, correlated, status Success)");
    } else {
        warn!("[SLS] LogoutResponse status not Success; local logout already completed");
    }

    debug!("[SLS] Redirecting browser to {target}");
    Redirect::to(target).into_response()
}

/// §7.7.2 correlation fields of a structurally valid LogoutResponse: the
/// `@InResponseTo` to consume and whether the status was `Success`.
#[derive(Debug)]
struct LogoutResponseFields {
    in_response_to: String,
    status_is_success: bool,
}

/// Parse the (already signature-verified) LogoutResponse XML and enforce the
/// §7.7.2 structural checks. Every element and attribute the §7.7.2 table gives
/// cardinality 1 is required: `samlp:LogoutResponse` root, `@Version` = 2.0, a
/// fresh `@IssueInstant`, `Issuer` = the RD, `@Destination` = our SLS endpoint,
/// and `@InResponseTo`. Returns the failure reason for the caller to log; every
/// failure resolves to the same "ignore and redirect" outcome, since the local
/// logout has already completed.
fn extract_logout_response_fields(
    saml_response: &str,
    rd_entity_id: &str,
    sls_url: &str,
) -> Result<LogoutResponseFields, String> {
    use crate::saml::{
        constants::{CLOCK_SKEW_SECONDS, MESSAGE_FRESHNESS_SECONDS},
        xml_parser::{self, find_child, find_descendant, inner_text},
    };
    use chrono::{DateTime, Duration, Utc};

    let doc = xml_parser::parse(saml_response)
        .map_err(|e| format!("could not parse LogoutResponse XML: {e}"))?;
    let root = doc.document_element();

    if doc.local_name(root) != Some("LogoutResponse") {
        return Err("response root is not samlp:LogoutResponse".to_string());
    }
    // eID §7.7.2 (cardinality 1): @Version MUST be 2.0.
    match doc.get_attribute(root, "Version") {
        Some("2.0") => {}
        Some(v) => return Err(format!("LogoutResponse has unsupported @Version {v:?}")),
        None => return Err("LogoutResponse is missing the required @Version".to_string()),
    }
    // eID §7.7.2 (cardinality 1): @IssueInstant MUST be present. A LogoutResponse
    // carries no Conditions, so bound it the same way the other envelopes are
    // bounded, otherwise a captured response stays structurally valid forever
    // (the InResponseTo consume-once check is the only other replay bound).
    let issue_instant = doc
        .get_attribute(root, "IssueInstant")
        .ok_or_else(|| "LogoutResponse is missing the required @IssueInstant".to_string())?;
    let issued: DateTime<Utc> = issue_instant
        .parse()
        .map_err(|_| format!("LogoutResponse has an invalid @IssueInstant: {issue_instant}"))?;
    let skew = Duration::seconds(CLOCK_SKEW_SECONDS);
    let now = Utc::now();
    if issued + Duration::seconds(MESSAGE_FRESHNESS_SECONDS) + skew < now {
        return Err(format!(
            "LogoutResponse is stale: issued at {issue_instant}"
        ));
    }
    if issued - skew > now {
        return Err(format!(
            "LogoutResponse @IssueInstant is in the future: {issue_instant}"
        ));
    }
    // Bind to the RD and our SLS endpoint, mirroring the ACS path.
    let issuer_ok = find_child(&doc, root, NS_SAML, "Issuer")
        .map(|n| inner_text(&doc, n))
        .is_some_and(|i| i.trim() == rd_entity_id);
    if !issuer_ok {
        return Err("LogoutResponse Issuer is not the RD".to_string());
    }
    // eID §7.7.2 (cardinality 1): @Destination MUST be present and MUST be our
    // SLS endpoint, so a response minted for another SP is not accepted here.
    match doc.get_attribute(root, "Destination") {
        Some(d) if d == sls_url => {}
        Some(_) => return Err("LogoutResponse @Destination is not our SLS endpoint".to_string()),
        None => return Err("LogoutResponse is missing the required @Destination".to_string()),
    }
    let Some(in_response_to) = doc.get_attribute(root, "InResponseTo") else {
        return Err("LogoutResponse has no InResponseTo".to_string());
    };

    let status_is_success = find_descendant(&doc, root, NS_SAMLP, "StatusCode")
        .and_then(|n| doc.get_attribute(n, "Value"))
        == Some(STATUS_SUCCESS);

    Ok(LogoutResponseFields {
        in_response_to: in_response_to.to_string(),
        status_is_success,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::handlers::test_support::MockAuthState;

    const RD: &str = "urn:test:rd";
    const SLS: &str = "https://dv.example.com/saml/sp/logout";

    fn logout_response_xml(
        root: &str,
        issuer: &str,
        in_response_to: Option<&str>,
        destination: Option<&str>,
        status: &str,
    ) -> String {
        logout_response_with_instant(
            root,
            issuer,
            in_response_to,
            destination,
            status,
            Some(&now_offset(chrono::Duration::zero())),
        )
    }

    fn now_offset(offset: chrono::Duration) -> String {
        (chrono::Utc::now() + offset)
            .format("%Y-%m-%dT%H:%M:%SZ")
            .to_string()
    }

    fn logout_response_with_instant(
        root: &str,
        issuer: &str,
        in_response_to: Option<&str>,
        destination: Option<&str>,
        status: &str,
        issue_instant: Option<&str>,
    ) -> String {
        let irt = in_response_to
            .map(|v| format!(r#" InResponseTo="{v}""#))
            .unwrap_or_default();
        let dest = destination
            .map(|v| format!(r#" Destination="{v}""#))
            .unwrap_or_default();
        let instant = issue_instant
            .map(|v| format!(r#" IssueInstant="{v}""#))
            .unwrap_or_default();
        format!(
            r#"<samlp:{root} xmlns:samlp="{NS_SAMLP}" xmlns:saml="{NS_SAML}" Version="2.0"{instant}{irt}{dest}>
                 <saml:Issuer>{issuer}</saml:Issuer>
                 <samlp:Status><samlp:StatusCode Value="{status}"/></samlp:Status>
               </samlp:{root}>"#
        )
    }

    #[test]
    fn extracts_fields_from_a_well_formed_logout_response() {
        let xml = logout_response_xml(
            "LogoutResponse",
            RD,
            Some("_req123"),
            Some(SLS),
            STATUS_SUCCESS,
        );
        let f = extract_logout_response_fields(&xml, RD, SLS).unwrap();
        assert!(f.status_is_success);
        assert_eq!(f.in_response_to, "_req123");
    }

    #[test]
    fn non_success_status_is_extracted_not_rejected() {
        // A non-Success status is reported, not a structural failure: the local
        // logout already completed either way.
        let xml = logout_response_xml(
            "LogoutResponse",
            RD,
            Some("_req123"),
            Some(SLS),
            "urn:oasis:names:tc:SAML:2.0:status:Responder",
        );
        let f = extract_logout_response_fields(&xml, RD, SLS).unwrap();
        assert!(!f.status_is_success);
    }

    #[test]
    fn rejects_wrong_root_issuer_destination_and_missing_in_response_to() {
        // Wrong root element.
        let xml = logout_response_xml("Response", RD, Some("_r"), Some(SLS), STATUS_SUCCESS);
        let err = extract_logout_response_fields(&xml, RD, SLS).unwrap_err();
        assert!(err.contains("not samlp:LogoutResponse"), "{err}");

        // Wrong issuer.
        let xml = logout_response_xml(
            "LogoutResponse",
            "urn:evil:idp",
            Some("_r"),
            Some(SLS),
            STATUS_SUCCESS,
        );
        let err = extract_logout_response_fields(&xml, RD, SLS).unwrap_err();
        assert!(err.contains("Issuer"), "{err}");

        // Mismatched Destination.
        let xml = logout_response_xml(
            "LogoutResponse",
            RD,
            Some("_r"),
            Some("https://attacker.example/sls"),
            STATUS_SUCCESS,
        );
        let err = extract_logout_response_fields(&xml, RD, SLS).unwrap_err();
        assert!(err.contains("Destination"), "{err}");

        // Missing InResponseTo.
        let xml = logout_response_xml("LogoutResponse", RD, None, Some(SLS), STATUS_SUCCESS);
        let err = extract_logout_response_fields(&xml, RD, SLS).unwrap_err();
        assert!(err.contains("InResponseTo"), "{err}");
    }

    #[test]
    fn absent_destination_is_rejected() {
        // eID §7.7.2 gives @Destination cardinality 1, so its absence is a
        // protocol violation rather than "nothing to compare".
        let xml = logout_response_xml("LogoutResponse", RD, Some("_r"), None, STATUS_SUCCESS);
        let err = extract_logout_response_fields(&xml, RD, SLS).unwrap_err();
        assert!(err.contains("missing the required @Destination"), "{err}");
    }

    #[test]
    fn rejects_missing_version_and_issue_instant() {
        // Both are cardinality 1 in the §7.7.2 table.
        let no_version =
            logout_response_xml("LogoutResponse", RD, Some("_r"), Some(SLS), STATUS_SUCCESS)
                .replace(r#" Version="2.0""#, "");
        let err = extract_logout_response_fields(&no_version, RD, SLS).unwrap_err();
        assert!(err.contains("missing the required @Version"), "{err}");

        let no_instant = logout_response_with_instant(
            "LogoutResponse",
            RD,
            Some("_r"),
            Some(SLS),
            STATUS_SUCCESS,
            None,
        );
        let err = extract_logout_response_fields(&no_instant, RD, SLS).unwrap_err();
        assert!(err.contains("missing the required @IssueInstant"), "{err}");
    }

    #[test]
    fn rejects_stale_and_future_issue_instant() {
        // Without a freshness bound a captured LogoutResponse would stay
        // structurally valid indefinitely.
        for (offset, expected) in [
            (-chrono::Duration::hours(1), "stale"),
            (chrono::Duration::hours(1), "in the future"),
        ] {
            let xml = logout_response_with_instant(
                "LogoutResponse",
                RD,
                Some("_r"),
                Some(SLS),
                STATUS_SUCCESS,
                Some(&now_offset(offset)),
            );
            let err = extract_logout_response_fields(&xml, RD, SLS).unwrap_err();
            assert!(err.contains(expected), "expected {expected}, got {err}");
        }
    }

    #[test]
    fn unparseable_xml_is_rejected() {
        let err = extract_logout_response_fields("not xml <<<", RD, SLS).unwrap_err();
        assert!(err.contains("could not parse"), "{err}");
    }

    // --- handler early-exit branches (no RD metadata loaded) ---------------

    #[tokio::test]
    async fn logout_without_active_session_redirects_home() {
        let mock = MockAuthState::empty();
        let resp = handle_logout(&mock, CookieJar::new(), "/").await;
        assert!(resp.status().is_redirection());
    }

    #[tokio::test]
    async fn logout_with_session_but_no_rd_metadata_completes_local_logout() {
        // A session exists (logout_session returns a NameID) but no RD descriptor
        // is loaded, so no SAML SLO is built: the browser is redirected home.
        let mut mock = MockAuthState::empty();
        mock.session = Some("_transient-name-id".to_string());
        let resp = handle_logout(&mock, CookieJar::new(), "/").await;
        assert!(resp.status().is_redirection());
    }

    #[tokio::test]
    async fn sls_without_saml_response_redirects() {
        let mock = MockAuthState::empty();
        let resp = handle_sls(
            crate::SamlLogoutPath,
            State(mock.clone()),
            State(mock.auth.clone()),
            Form(HashMap::new()),
        )
        .await;
        assert!(resp.status().is_redirection());
    }

    #[tokio::test]
    async fn sls_with_invalid_base64_redirects() {
        // Malformed input is dropped like any other invalid LogoutResponse: the
        // local logout already happened, so the user is never stranded.
        let mock = MockAuthState::empty();
        let mut params = HashMap::new();
        params.insert("SAMLResponse".to_string(), "!!!not-base64!!!".to_string());
        let resp = handle_sls(
            crate::SamlLogoutPath,
            State(mock.clone()),
            State(mock.auth.clone()),
            Form(params),
        )
        .await;
        assert!(resp.status().is_redirection());
    }

    #[tokio::test]
    async fn sls_without_rd_metadata_redirects() {
        // Valid base64 body, but no RD descriptor to verify against: drop and
        // redirect (the local logout already happened in `handle_logout`).
        let mock = MockAuthState::empty();
        let mut params = HashMap::new();
        params.insert(
            "SAMLResponse".to_string(),
            BASE64.encode(b"<samlp:LogoutResponse/>"),
        );
        let resp = handle_sls(
            crate::SamlLogoutPath,
            State(mock.clone()),
            State(mock.auth.clone()),
            Form(params),
        )
        .await;
        assert!(resp.status().is_redirection());
    }
}
