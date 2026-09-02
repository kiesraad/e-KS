//! ACS: resolve the SAML artifact into validated claims and hand off to the
//! embedding application.
//!
//! Every failure resolving the artifact maps to a small `AuthFailure`. Rather
//! than render the user-facing page inline on the ACS URL (which carries the
//! one-time `SAMLart` in its query), the handler 303-redirects to the
//! query-clean error endpoint below, and only *that* endpoint asks the
//! embedding application to render the page (TVS T3/L10). This keeps the
//! artifact out of the address bar, history, and any `Referer` the error page
//! emits. The technical detail is logged at the failure site.
use crate::{
    LoginErrorPath, SamlAcsPath,
    bindings::soap::{send_soap_request, unwrap_soap},
    config::AuthConfig,
    saml::{
        decryption::DecryptionKey,
        idp_metadata::IdpMetadata,
        loa::MINIMUM_LOA,
        messages::{CreatedMessage, create_artifact_resolve},
        validation::{
            Claims, ValidateArtifactResponseOpts, ValidateAssertionOpts, ValidateResponseOpts,
            validate_artifact_response_at, validate_assertion_at, validate_response_at,
        },
        xml_builder::wrap_in_soap_envelope,
        xml_parser::{Document, NodeId, parse},
    },
    state::{AuthFailure, AuthServiceState, AuthState},
};
use axum::{
    extract::{FromRef, Query, State},
    http::{HeaderMap, HeaderValue, header},
    response::{IntoResponse, Redirect, Response},
};
use axum_extra::{extract::CookieJar, routing::TypedPath};
use std::collections::HashMap;
use tracing::{debug, error, info, warn};

/// Assertion Consumer Service (eID §7.1 steps 4-8 / §3.1.1).
///
/// Receives the artifact via HTTP-Artifact binding (eID §7.4), resolves it over
/// the mTLS back-channel (eID §7.5, §9.4), validates the ArtifactResponse
/// (§7.6.1), Response (§7.6.2), and Assertion (§7.6.3, §7.6.3.5). On success,
/// delegates to the embedding application via `AuthState::on_authenticated` so
/// it can create its own session and set the appropriate cookie.
pub async fn handle_acs<S>(
    _: SamlAcsPath,
    State(state): State<S>,
    State(auth_state): State<AuthServiceState>,
    jar: CookieJar,
    headers: HeaderMap,
    Query(params): Query<HashMap<String, String>>,
) -> Response
where
    S: AuthState,
    AuthServiceState: FromRef<S>,
{
    debug!("[ACS] Handler entered, query params: {}", params.len());

    let claims = match resolve_artifact_to_claims(&auth_state, &params).await {
        Ok(c) => c,
        // Technical detail is logged at the failure site; the query-clean error
        // endpoint renders the user-facing page (TVS T3/L10).
        Err(failure) => return fail_redirect(failure, jar),
    };

    let jar = match confirm_pending_request(&state, &auth_state, &claims, jar, &headers).await {
        Ok(jar) => jar,
        Err(jar) => return fail_redirect(AuthFailure::Error, jar),
    };

    // SECURITY: never log decrypted SubjectID values; they are PII (BSN /
    // pseudonym per eID §7.6.3.4). Log only non-PII metadata for tracing.
    info!(
        "[ACS] Authentication successful. acting_subject_present={}, \
         legal_subject_present={}, loa={:?}, authenticating_authority={:?}, \
         service_uuid_present={}",
        claims.acting_subject_id.is_some(),
        claims.legal_subject_id.is_some(),
        claims.authn_context_class_ref.as_deref(),
        claims.authenticating_authority.as_deref(),
        claims.service_uuid.is_some(),
    );

    // Hand off to the embedding application to create its session. An
    // assertion without an acting SubjectID carries no usable identity, so it
    // is treated as an authentication failure (TVS L10) rather than handed on,
    // guaranteeing the application's `on_authenticated` a SubjectID.
    let Some(subject_id) = claims.acting_subject_id else {
        warn!("[ACS] No acting SubjectID in validated assertion: treating as auth failure");
        return fail_redirect(AuthFailure::Error, jar);
    };
    debug!("[ACS] Handing off to AuthState::on_authenticated");
    state
        .on_authenticated(subject_id, claims.name_id, jar, &headers)
        .await
}

/// Require the validated assertion to answer an AuthnRequest this DV issued
/// from this browser. `Err` carries the cleared jar for the caller's failure
/// redirect (TVS L10); every rejection here is an `AuthFailure::Error`.
///
/// eID §7.6.3.5 rule 4 / §9.7: the Assertion must be a response to an
/// AuthnRequest this DV actually issued, and the matched ID is consumed in the
/// same atomic step so a replay of the same Assertion can never be accepted
/// (the store is the application's, so this holds even when /login and the ACS
/// callback are served by different instances). Fails closed: an absent,
/// unknown, expired, already-consumed, or unverifiable InResponseTo is
/// rejected. The one-shot flow cookie is cleared on the returned jar either way.
async fn confirm_pending_request<S: AuthState>(
    state: &S,
    auth_state: &AuthServiceState,
    claims: &Claims,
    jar: CookieJar,
    headers: &HeaderMap,
) -> Result<CookieJar, CookieJar> {
    let Some(in_response_to) = claims.in_response_to.clone() else {
        warn!("[ACS] Assertion has no InResponseTo: cannot correlate to a pending AuthnRequest");
        return Err(jar);
    };

    // Login-CSRF / forced-login defense: this ACS callback MUST come from the
    // browser that started the flow. The cookie set by `/login` must be present
    // and bound to this AuthnRequest ID (the assertion's InResponseTo) and the
    // same User-Agent. The cookie is cleared (one-shot) regardless of the outcome.
    let (flow_ok, jar) = crate::handlers::flow::verify_and_clear(
        jar,
        &auth_state.auth_config().dv.acs_url,
        &in_response_to,
        headers,
    );
    if !flow_ok {
        warn!(
            "[ACS] SSO flow cookie missing or not bound to this AuthnRequest/User-Agent: \
             rejecting (possible login CSRF / forced login)"
        );
        return Err(jar);
    }

    if !state.consume_if_pending(in_response_to).await {
        warn!(
            "[ACS] InResponseTo did not match an outstanding AuthnRequest \
             (unknown, expired, or replayed): rejecting"
        );
        return Err(jar);
    }
    Ok(jar)
}

/// Query-clean landing for a failed SAML authentication: the redirect target of
/// the [`handle_acs`] failure paths ([`LoginErrorPath`]).
///
/// Maps the non-sensitive `reason` code back to an [`AuthFailure`] and delegates
/// the user-facing page to the embedding application. Because this URL never
/// carries `SAMLart`, the one-time artifact stays out of the browser address
/// bar, history, and `Referer`.
pub async fn handle_login_error<S>(
    _: LoginErrorPath,
    State(state): State<S>,
    jar: CookieJar,
    headers: HeaderMap,
    Query(params): Query<HashMap<String, String>>,
) -> Response
where
    S: AuthState,
{
    let failure = params
        .get("reason")
        .map_or(AuthFailure::Error, |r| failure_from_reason(r));
    let mut response = state.on_authentication_failed(failure, jar, &headers).await;
    harden_headers(response.headers_mut());
    response
}

/// Redirect a failed ACS callback to the query-clean error endpoint.
///
/// Only the non-sensitive reason code is carried forward; the one-time `SAMLart`
/// (and any other query) is dropped from the browser's address bar and history.
/// Cookie changes staged on `jar` (the one-shot flow-cookie clearing) ride the
/// 303 response so they are still applied. The root-absolute target relies on
/// the embedding application merging [`router`](crate::router) at the root.
fn fail_redirect(failure: AuthFailure, jar: CookieJar) -> Response {
    let target = format!(
        "{}?reason={}",
        LoginErrorPath::PATH,
        failure_reason(failure)
    );
    let mut response = (jar, Redirect::to(&target)).into_response();
    harden_headers(response.headers_mut());
    response
}

/// Stable, non-sensitive reason code for the error-redirect URL. Never contains
/// PII or the one-time artifact, so it is safe to expose in the browser address
/// bar, history, and `Referer`.
fn failure_reason(failure: AuthFailure) -> &'static str {
    match failure {
        AuthFailure::Cancelled => "cancelled",
        AuthFailure::Error => "error",
        AuthFailure::Unavailable => "unavailable",
    }
}

/// Parse a reason code back into a failure. Anything unrecognized (a missing,
/// tampered, or unknown value) falls back to the generic `Error` page, so the
/// error endpoint fails safe.
fn failure_from_reason(reason: &str) -> AuthFailure {
    match reason {
        "cancelled" => AuthFailure::Cancelled,
        "unavailable" => AuthFailure::Unavailable,
        _ => AuthFailure::Error,
    }
}

/// Defense-in-depth headers for the failed-authentication path: never cache a URL
/// that carried the artifact, and never leak it (or the error URL) via `Referer`.
fn harden_headers(headers: &mut HeaderMap) {
    headers.insert(header::CACHE_CONTROL, HeaderValue::from_static("no-store"));
    headers.insert(
        header::REFERRER_POLICY,
        HeaderValue::from_static("no-referrer"),
    );
}

/// Pull the one-time-use artifact out of the ACS query string (eID §7.4).
///
/// The artifact is an opaque reference, so a short prefix is safe to log for
/// correlation and is not itself sensitive PII.
fn artifact_from_params(params: &HashMap<String, String>) -> Result<&String, AuthFailure> {
    let Some(artifact) = params.get("SAMLart") else {
        warn!("[ACS] Missing SAMLart query parameter");
        return Err(AuthFailure::Error);
    };

    info!(
        "[ACS] Artifact received: {}... (len={})",
        artifact.chars().take(20).collect::<String>(),
        artifact.len()
    );

    Ok(artifact)
}

/// Parse the SOAP ArtifactResponse envelope exactly once; the whole
/// ArtifactResponse -> Response -> Assertion chain is then navigated on this one
/// tree, so inner elements keep the namespaces they inherit.
fn parse_soap_envelope(soap: &str) -> Result<Document<'_>, AuthFailure> {
    parse(soap).map_err(|e| {
        error!("[ACS] Failed to parse SOAP ArtifactResponse: {e}");
        AuthFailure::Error
    })
}

/// Resolve the artifact into validated [`Claims`], or an [`AuthFailure`] the
/// caller turns into a user-facing page.
async fn resolve_artifact_to_claims(
    auth_state: &AuthServiceState,
    params: &HashMap<String, String>,
) -> Result<Claims, AuthFailure> {
    let artifact = artifact_from_params(params)?;

    let cfg = auth_state.auth_config();
    let dv_keys = auth_state.dv_keys();
    // Without the RD descriptor we have neither the ARS endpoint to resolve the
    // artifact against nor the RD signing keys to verify the response (eID §9.2),
    // so the flow cannot proceed. Transient: the RD metadata is not loaded yet.
    let Some(rd) = auth_state.rd_metadata() else {
        warn!("[ACS] RD metadata not loaded: cannot resolve or validate the artifact");
        return Err(AuthFailure::Unavailable);
    };

    debug!(
        "[ACS] Using DV entity_id={}, ARS url={}, signing_keys={}, encryption_keys={}",
        cfg.dv.entity_id,
        rd.ars_url,
        dv_keys.signing.len(),
        dv_keys.encryption.len()
    );

    // 1. Create signed ArtifactResolve (eID §7.5)
    let resolve = build_artifact_resolve(artifact, cfg, &rd, dv_keys.primary_signing())?;

    // 2. Wrap in SOAP and send to ARS over mTLS (eID §9.4)
    let soap_response = send_artifact_resolve(&resolve.xml, &rd, cfg).await?;

    // 3-5. Parse the SOAP ArtifactResponse exactly once and validate the
    //      ArtifactResponse -> Response -> Assertion chain by navigating that
    //      single tree. Inner elements (Response, Assertion) inherit their
    //      namespaces from the ArtifactResponse and are never re-parsed as
    //      standalone fragments; signature verification uses the self-contained
    //      source bytes of the RD-signed ArtifactResponse element.
    let doc = parse_soap_envelope(&soap_response)?;
    let chain = ResponseChain {
        doc: &doc,
        auth_state,
        rd: &rd,
    };
    chain.claims(&resolve.id)
}

/// The parsed SOAP document plus the trust context threaded through the
/// ArtifactResponse -> Response -> Assertion validation chain (eID §7.6).
struct ResponseChain<'a, 'input> {
    doc: &'a Document<'input>,
    auth_state: &'a AuthServiceState,
    rd: &'a IdpMetadata,
}

impl ResponseChain<'_, '_> {
    /// Validate the full chain and extract the [`Claims`]. `resolve_id` is the
    /// `@ID` of the ArtifactResolve this response must answer.
    fn claims(&self, resolve_id: &str) -> Result<Claims, AuthFailure> {
        let root = self.doc.document_element();
        let Some(art_node) = unwrap_soap(self.doc, root) else {
            warn!(
                "[ACS] Failed to unwrap SOAP envelope (root_element={:?})",
                self.doc.local_name(root)
            );
            return Err(AuthFailure::Error);
        };

        // 3. Validate ArtifactResponse (eID §7.6.1) using RD signing certs from metadata
        let response_node = self.response_node(art_node, resolve_id)?;

        // 4. Validate Response (eID §7.6.2): handle cancellation / IdP errors
        let assertion_node = self.assertion_node(response_node)?;

        // 5. Validate Assertion (eID §7.6.3, §7.6.3.5). The Assertion is
        //    authenticated by the enveloping RD signature on the ArtifactResponse
        //    (verified in step 3); per §9.1 only signatures outside an
        //    Assertion/Advice are validated. Binds the Assertion Issuer to the
        //    RD EntityID (`minvws/nl-rdo-max`).
        let claims = self.assertion_claims(assertion_node)?;

        self.check_matching_in_response_to(response_node, &claims)?;
        Ok(claims)
    }

    // Cross-check only: both values come from the same Response, so this does
    // NOT by itself satisfy eID §7.6.3.5 rule 4. Rule 4 (the assertion answers an
    // AuthnRequest this DV actually issued) is enforced in
    // `confirm_pending_request` below, which matches `claims.in_response_to`
    // against the pending-request store and consumes it.
    //
    // What this adds on top: eID §7.6.2 gives the Response an @InResponseTo of
    // cardinality 1, and it names the same AuthnRequest as the assertion's
    // SubjectConfirmationData. Since only the assertion's value is checked
    // against the store, requiring the two to agree rejects a Response whose
    // envelope and assertion name different requests, i.e. an assertion spliced
    // into a Response for another flow.
    fn check_matching_in_response_to(
        &self,
        response_node: NodeId,
        claims: &Claims,
    ) -> Result<(), AuthFailure> {
        let response_in_response_to = self.doc.get_attribute(response_node, "InResponseTo");
        if response_in_response_to != claims.in_response_to.as_deref() {
            warn!(
                "[ACS] Response @InResponseTo does not match the assertion's InResponseTo: rejecting"
            );
            return Err(AuthFailure::Error);
        }
        Ok(())
    }
}

fn build_artifact_resolve(
    artifact: &str,
    cfg: &AuthConfig,
    rd: &IdpMetadata,
    signing_key: &crate::keys::KeyPair,
) -> Result<CreatedMessage, AuthFailure> {
    debug!("[ACS] Step 1: building signed ArtifactResolve");
    match create_artifact_resolve(artifact, &cfg.dv.entity_id, &rd.ars_url, signing_key) {
        Ok(m) => {
            debug!(
                "[ACS] ArtifactResolve built: id={}, xml_len={}",
                m.id,
                m.xml.len()
            );
            Ok(m)
        }
        Err(e) => {
            error!("[ACS] Failed to create ArtifactResolve: {e}");
            Err(AuthFailure::Error)
        }
    }
}

async fn send_artifact_resolve(
    resolve_xml: &str,
    rd: &IdpMetadata,
    cfg: &AuthConfig,
) -> Result<String, AuthFailure> {
    debug!("[ACS] Step 2: sending ArtifactResolve over mTLS SOAP back-channel");
    let soap_xml = wrap_in_soap_envelope(resolve_xml).map_err(|e| {
        error!("[ACS] Failed to build SOAP envelope: {e}");
        AuthFailure::Error
    })?;
    match send_soap_request(&rd.ars_url, &soap_xml, &cfg.tls).await {
        Ok(r) => {
            debug!(
                "[ACS] SOAP back-channel returned response (len={})",
                r.len()
            );
            Ok(r)
        }
        Err(e) => {
            error!("[ACS] SOAP back-channel failed: {e}");
            Err(AuthFailure::Error)
        }
    }
}

impl ResponseChain<'_, '_> {
    fn response_node(&self, art_node: NodeId, expected_id: &str) -> Result<NodeId, AuthFailure> {
        debug!(
            "[ACS] Step 3: validating ArtifactResponse against {} RD signing key(s), \
             expected InResponseTo={}",
            self.rd.signing_keys.len(),
            expected_id
        );
        let mut errors = Vec::new();
        let response = validate_artifact_response_at(
            self.doc,
            art_node,
            &ValidateArtifactResponseOpts {
                trusted_keys: &self.rd.signing_keys,
                expected_in_response_to: expected_id,
                // eID §7.6.1: bind the ArtifactResponse Issuer to the RD EntityID.
                expected_issuer: Some(&self.rd.entity_id),
            },
            &mut errors,
        );
        if !errors.is_empty() {
            error!("[ACS] ArtifactResponse validation failed: {errors:?}");
            return Err(AuthFailure::Error);
        }
        debug!("[ACS] Step 3: ArtifactResponse OK");
        response.ok_or_else(|| {
            warn!("[ACS] No Response in ArtifactResponse");
            AuthFailure::Error
        })
    }

    fn assertion_node(&self, response_node: NodeId) -> Result<NodeId, AuthFailure> {
        debug!("[ACS] Step 4: validating inner Response status");
        let mut errors = Vec::new();
        let assertion = validate_response_at(
            self.doc,
            response_node,
            // eID §7.6.2: bind the Response to this DV's ACS and the RD as issuer,
            // mirroring the assertion-level Recipient/Issuer checks (§7.6.3.5 r1-2).
            &ValidateResponseOpts {
                expected_destination: Some(&self.auth_state.auth_config().dv.acs_url),
                expected_issuer: Some(&self.rd.entity_id),
            },
            &mut errors,
        );

        if !errors.is_empty() {
            let errors_str = errors.join("; ");
            let is_authn_failed = errors_str.contains("AuthnFailed");
            let is_cancelled = errors_str.contains("Authentication cancelled");

            // TVS "Checklist Testen" v2.1 T3: the user cancelled.
            if is_authn_failed || is_cancelled {
                warn!("[ACS] Authentication cancelled by user");
                return Err(AuthFailure::Cancelled);
            }

            // TVS "Checklist Testen" v2.1 L10: RD/DigiD error status.
            warn!("[ACS] Authentication failed: {errors_str}");
            return Err(AuthFailure::Error);
        }
        debug!("[ACS] Step 4: Response status Success");

        assertion.ok_or_else(|| {
            warn!("[ACS] No Assertion element extracted from successful Response");
            AuthFailure::Error
        })
    }

    fn assertion_claims(&self, assertion_node: NodeId) -> Result<Claims, AuthFailure> {
        debug!("[ACS] Step 5: validating Assertion");

        let cfg = self.auth_state.auth_config();
        let priv_keys: Vec<DecryptionKey<'_>> = self
            .auth_state
            .dv_keys()
            .encryption
            .iter()
            .map(|k| DecryptionKey {
                key_pem: &k.key_pem,
                key_name: k.key_name.as_str(),
            })
            .collect();

        let mut errors = Vec::new();
        let claims = validate_assertion_at(
            self.doc,
            assertion_node,
            &ValidateAssertionOpts {
                dv_entity_id: &cfg.dv.entity_id,
                expected_recipient: &cfg.dv.acs_url,
                // eID §9.1: the Assertion is authenticated by the enveloping RD
                // signature on the ArtifactResponse (verified in step 3); here we
                // only bind the Assertion Issuer to the RD EntityID (`minvws/nl-rdo-max`).
                expected_issuer: Some(&self.rd.entity_id),
                private_keys: &priv_keys,
                minimum_loa: Some(MINIMUM_LOA),
                // eID §7.6.3.4: bind to the registered service.
                expected_service_uuid: Some(&cfg.dv.service_uuid),
            },
            &mut errors,
        );

        claims.ok_or_else(|| {
            error!("[ACS] Assertion validation failed: {errors:?}");
            AuthFailure::Error
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{config::AuthConfig, handlers::test_support::MockAuthState};

    fn load_signing_key() -> crate::keys::KeyPair {
        let dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("fixtures");
        let cfg = AuthConfig::default().with_certs_dir(dir);
        crate::keys::load_key_set(&cfg.dv.signing, &cfg.dv.encryption)
            .expect("load fixtures")
            .signing
            .remove(0)
    }

    fn rd_metadata() -> IdpMetadata {
        IdpMetadata {
            entity_id: "urn:test:rd".to_string(),
            sso_url: "https://rd.example.com/sso".to_string(),
            ars_url: "https://rd.example.com/artifact".to_string(),
            slo_url: "https://rd.example.com/slo".to_string(),
            signing_keys: Vec::new(),
            cache_duration: None,
        }
    }

    /// The `Location` a failed ACS callback redirected to, or `None` if the
    /// response was not a redirect. Also asserts the artifact-hardening headers.
    fn redirect_location(resp: &Response) -> Option<String> {
        assert_eq!(
            resp.headers().get(axum::http::header::CACHE_CONTROL),
            Some(&axum::http::HeaderValue::from_static("no-store")),
            "failure responses must not be cached"
        );
        assert_eq!(
            resp.headers().get(axum::http::header::REFERRER_POLICY),
            Some(&axum::http::HeaderValue::from_static("no-referrer")),
            "failure responses must not leak the artifact via Referer"
        );
        resp.headers()
            .get(axum::http::header::LOCATION)
            .and_then(|v| v.to_str().ok())
            .map(str::to_owned)
    }

    #[tokio::test]
    async fn missing_saml_artifact_redirects_to_clean_error() {
        // No `SAMLart` query parameter: resolve_artifact_to_claims fails closed with Error,
        // and the handler 303-redirects to the query-clean error endpoint so
        // the artifact-bearing URL is never rendered in the browser.
        let mock = MockAuthState::empty();
        let resp = handle_acs(
            SamlAcsPath,
            State(mock.clone()),
            State(mock.auth.clone()),
            axum_extra::extract::CookieJar::new(),
            HeaderMap::new(),
            Query(HashMap::new()),
        )
        .await;
        assert_eq!(resp.status(), axum::http::StatusCode::SEE_OTHER);
        let location = redirect_location(&resp).expect("redirect Location header");
        assert!(
            location.ends_with("/login/error?reason=error"),
            "{location}"
        );
        assert!(
            !location.contains("SAMLart"),
            "artifact must not leak: {location}"
        );
    }

    #[tokio::test]
    async fn artifact_without_rd_metadata_redirects_as_unavailable() {
        // An artifact is present but no RD descriptor is loaded, so the flow
        // cannot be resolved or validated: redirect to the error endpoint with
        // the `unavailable` reason (and the artifact stripped from the target).
        let mock = MockAuthState::empty();
        let mut params = HashMap::new();
        params.insert(
            "SAMLart".to_string(),
            "AAQAA-some-opaque-artifact".to_string(),
        );
        let resp = handle_acs(
            SamlAcsPath,
            State(mock.clone()),
            State(mock.auth.clone()),
            axum_extra::extract::CookieJar::new(),
            HeaderMap::new(),
            Query(params),
        )
        .await;
        assert_eq!(resp.status(), axum::http::StatusCode::SEE_OTHER);
        let location = redirect_location(&resp).expect("redirect Location header");
        assert!(
            location.ends_with("/login/error?reason=unavailable"),
            "{location}"
        );
        assert!(
            !location.contains("SAMLart"),
            "artifact must not leak: {location}"
        );
    }

    #[tokio::test]
    async fn error_endpoint_maps_reason_and_hardens_headers() {
        // The error endpoint renders the embedder's failure page for the given
        // reason, with the same no-store / no-referrer hardening.
        let mock = MockAuthState::empty();
        let mut params = HashMap::new();
        params.insert("reason".to_string(), "cancelled".to_string());
        let resp = handle_login_error(
            LoginErrorPath,
            State(mock.clone()),
            axum_extra::extract::CookieJar::new(),
            HeaderMap::new(),
            Query(params),
        )
        .await;
        // MockAuthState renders Cancelled as 403.
        assert_eq!(resp.status(), axum::http::StatusCode::FORBIDDEN);
        // Not a redirect, but still hardened.
        assert_eq!(
            resp.headers().get(axum::http::header::CACHE_CONTROL),
            Some(&axum::http::HeaderValue::from_static("no-store"))
        );
        assert_eq!(
            resp.headers().get(axum::http::header::REFERRER_POLICY),
            Some(&axum::http::HeaderValue::from_static("no-referrer"))
        );
    }

    #[tokio::test]
    async fn error_endpoint_unknown_reason_falls_back_to_error() {
        // A missing or tampered reason must fail safe to the generic error page.
        let mock = MockAuthState::empty();
        let mut params = HashMap::new();
        params.insert("reason".to_string(), "bogus".to_string());
        let resp = handle_login_error(
            LoginErrorPath,
            State(mock.clone()),
            axum_extra::extract::CookieJar::new(),
            HeaderMap::new(),
            Query(params),
        )
        .await;
        // MockAuthState renders Error as 401.
        assert_eq!(resp.status(), axum::http::StatusCode::UNAUTHORIZED);
    }

    #[test]
    fn build_artifact_resolve_produces_a_signed_message() {
        let cfg = AuthConfig {
            dv: crate::config::DvConfig {
                entity_id: "urn:test:dv".to_string(),
                ..Default::default()
            },
            ..AuthConfig::default()
        };
        let rd = rd_metadata();
        let key = load_signing_key();

        let msg = build_artifact_resolve("AAQAA-artifact", &cfg, &rd, &key)
            .expect("ArtifactResolve must build and sign");
        assert!(msg.id.starts_with('_'), "message id: {}", msg.id);
        assert!(msg.xml.contains("ArtifactResolve"), "{}", msg.xml);
        // The artifact and the destination ARS endpoint are carried in the XML.
        assert!(msg.xml.contains("AAQAA-artifact"));
        assert!(msg.xml.contains("https://rd.example.com/artifact"));
    }
}
