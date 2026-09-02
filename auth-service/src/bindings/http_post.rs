use askama::Template;
use axum::{
    http::{HeaderValue, header},
    response::{IntoResponse, Response},
};
use axum_extra::routing::TypedPath;
use base64::{Engine, engine::general_purpose::STANDARD as BASE64};

use crate::{error::Result, types::EndpointUrl};

/// Body of the auto-submit script. Served verbatim at
/// [`AutosubmitJsPath`](crate::AutosubmitJsPath).
/// Kept external (rather than inline) so the page complies with a strict CSP
/// that does not permit `script-src 'unsafe-inline'`.
pub const AUTOSUBMIT_JS: &str = "document.getElementById('saml').submit();\n";

/// Inputs to [`create_post_form`]. All values are HTML-escaped by askama, so an
/// attacker-influenced value (e.g. a poisoned IdP endpoint from metadata)
/// cannot break out of its attribute and inject markup.
#[derive(Template)]
#[template(path = "post_form.html")]
struct PostFormTemplate<'a> {
    destination: &'a str,
    param_name: &'a str,
    encoded: &'a str,
    autosubmit_js_path: &'a str,
}

/// Build the Content-Security-Policy for the auto-submit page: the strict
/// default ([`crate::csp`]) with `form-action` widened by exactly `action_url`
/// (the IdP endpoint).
///
/// The default `form-action 'self'` would otherwise block the POST to the IdP.
/// This per-response policy is set by the login/logout handlers and takes
/// precedence over the router-wide default (the outer layer only sets the
/// header if it is not already present).
///
/// SECURITY: the widened value is metadata-derived, so it must not be able to
/// smuggle in a directive of its own. Two things prevent that: `action_url` is
/// validated upstream in [`crate::saml::idp_metadata`] to reject `;`, quotes
/// and whitespace, and `script-src` is emitted before `form-action` (see the
/// ordering note on `csp::DEFAULT`), so a second `script-src` would lose to
/// ours under CSP's first-occurrence-wins rule.
pub fn autosubmit_csp(action_url: &EndpointUrl) -> String {
    crate::csp::ContentSecurityPolicy::strict()
        .widen("form-action", action_url.as_str())
        .to_string()
}

/// Generate the HTML auto-submit form for HTTP-POST binding.
///
/// `destination` (the IdP endpoint) and `param_name` are HTML-escaped by the
/// template; `encoded` is base64 (no HTML-special characters).
pub fn create_post_form(
    destination: &EndpointUrl,
    saml_xml: &str,
    param_name: &str,
) -> Result<String> {
    Ok(PostFormTemplate {
        destination: destination.as_str(),
        param_name,
        encoded: &BASE64.encode(saml_xml.as_bytes()),
        autosubmit_js_path: crate::AutosubmitJsPath::PATH,
    }
    .render()?)
}

/// Build the autosubmit HTTP-POST response that carries `saml_xml` (under
/// `param_name`) to `action_url`. Used by both the login and logout handlers,
/// which differ only in the cookie jar they prepend to it.
///
/// The page holds a signed SAML message, so it is served with a per-response
/// CSP whitelisting `action_url` (see [`autosubmit_csp`]) plus `no-store` and
/// `DENY` framing as defense-in-depth.
pub fn autosubmit_post_response(
    action_url: &EndpointUrl,
    saml_xml: &str,
    param_name: &str,
) -> Result<Response> {
    let html = create_post_form(action_url, saml_xml, param_name)?;
    // The IdP URL is validated upstream ([`crate::saml::idp_metadata`] rejects
    // `;`, quotes and whitespace), so a URL that is not representable in a
    // header value means something is badly wrong; refuse to serve the page
    // rather than degrade its CSP.
    let csp_value = HeaderValue::from_str(&autosubmit_csp(action_url))?;
    Ok((
        [
            (header::CONTENT_TYPE, HeaderValue::from_static("text/html")),
            (header::CONTENT_SECURITY_POLICY, csp_value),
            (header::CACHE_CONTROL, HeaderValue::from_static("no-store")),
            (header::X_FRAME_OPTIONS, HeaderValue::from_static("DENY")),
        ],
        html,
    )
        .into_response())
}

#[cfg(test)]
mod tests {
    // The tests slice their own ASCII fixtures to pull a value out of the
    // rendered HTML; the production ban on string slicing does not apply.
    #![allow(clippy::string_slice)]

    use super::*;

    fn idp(url: &str) -> EndpointUrl {
        EndpointUrl::from_metadata(url, "SSO").expect("test IdP endpoint")
    }

    #[test]
    fn form_contains_destination_and_encoded_value() {
        let html = create_post_form(
            &idp("https://idp.example.com/sso"),
            "<xml>test</xml>",
            "SAMLRequest",
        )
        .unwrap();
        assert!(html.contains(r#"action="https://idp.example.com/sso""#));
        assert!(html.contains(r#"name="SAMLRequest""#));
        let expected_b64 = BASE64.encode(b"<xml>test</xml>");
        assert!(html.contains(&expected_b64));
    }

    #[test]
    fn form_round_trips_via_base64() {
        let original = "<samlp:AuthnRequest>data</samlp:AuthnRequest>";
        let html = create_post_form(&idp("https://x.com"), original, "SAMLRequest").unwrap();
        let marker = r#"value=""#;
        let start = html.find(marker).unwrap() + marker.len();
        let end = start + html[start..].find('"').unwrap();
        let encoded = &html[start..end];
        let decoded = String::from_utf8(BASE64.decode(encoded).unwrap()).unwrap();
        assert_eq!(decoded, original);
    }

    #[test]
    fn form_uses_external_script_not_inline() {
        let html =
            create_post_form(&idp("https://idp.example.com/sso"), "<xml/>", "SAMLRequest").unwrap();
        assert!(html.contains(&format!(
            r#"<script src="{p}""#,
            p = crate::AutosubmitJsPath::PATH
        )));
        assert!(
            !html.contains(".submit()"),
            "form must not contain inline script"
        );
    }

    #[test]
    fn autosubmit_csp_whitelists_action_url() {
        let csp = autosubmit_csp(&idp("https://idp.example.com/sso"));
        assert!(csp.contains("form-action 'self' https://idp.example.com/sso"));
        assert!(csp.contains("script-src 'self'"));
        // Only `form-action` is widened: the page keeps the global lockdown.
        assert!(csp.contains("default-src 'none'"));
        assert!(csp.contains("base-uri 'none'"));
        assert!(csp.contains("frame-ancestors 'none'"));
        assert!(csp.contains("require-trusted-types-for 'script'"));
        assert!(csp.contains("trusted-types 'none'"));
        assert!(!csp.contains("unsafe-inline") && !csp.contains("unsafe-eval"));
    }

    #[test]
    fn autosubmit_csp_script_src_precedes_form_action() {
        // CSP first-occurrence-wins: our script-src must come before the
        // form-action directive that carries the (metadata-derived) URL, so a
        // smuggled `; script-src 'unsafe-inline'` could never displace it.
        let csp = autosubmit_csp(&idp("https://idp.example.com/sso"));
        let script_pos = csp.find("script-src 'self'").unwrap();
        let form_pos = csp.find("form-action").unwrap();
        assert!(script_pos < form_pos);
    }

    #[test]
    fn destination_that_could_break_out_of_the_attribute_is_unrepresentable() {
        // A poisoned IdP endpoint never reaches the form: `EndpointUrl` refuses
        // the quote and angle brackets that would break out of action="...",
        // so `create_post_form` cannot be called with one.
        assert!(
            EndpointUrl::from_metadata(r#"https://x/"><script>alert(1)</script>"#, "SSO").is_err()
        );
    }

    #[test]
    fn create_post_form_emits_the_autosubmit_script_tag() {
        let html =
            create_post_form(&idp("https://idp.example.com/sso"), "<xml/>", "SAMLRequest").unwrap();
        assert!(html.contains(&format!(
            r#"<script src="{p}""#,
            p = crate::AutosubmitJsPath::PATH
        )));
    }

    #[test]
    fn create_post_form_escapes_ampersand_and_apostrophe() {
        // Covers the `&` and `'` escape branches that the breakout test does not
        // exercise (a `&` in a query string, an apostrophe in the value).
        let html = create_post_form(
            &idp("https://idp.example.com/sso?a=1&b=2"),
            "it's <xml/>",
            "SAML'Request",
        )
        .unwrap();
        assert!(html.contains("https://idp.example.com/sso?a=1&#38;b=2"));
        assert!(html.contains("SAML&#39;Request"));
        // The raw ampersand / apostrophe never reach the markup unescaped.
        assert!(!html.contains("a=1&b=2"));
        assert!(!html.contains("SAML'Request"));
    }

    #[test]
    fn autosubmit_post_response_sets_security_headers() {
        use axum::http::header;

        let resp = autosubmit_post_response(
            &idp("https://idp.example.com/sso"),
            "<samlp:AuthnRequest/>",
            "SAMLRequest",
        )
        .unwrap();
        let headers = resp.headers();

        assert_eq!(headers.get(header::CONTENT_TYPE).unwrap(), "text/html");
        assert_eq!(headers.get(header::CACHE_CONTROL).unwrap(), "no-store");
        assert_eq!(headers.get(header::X_FRAME_OPTIONS).unwrap(), "DENY");
        // The per-response CSP whitelists the IdP endpoint as a form target.
        let csp = headers
            .get(header::CONTENT_SECURITY_POLICY)
            .unwrap()
            .to_str()
            .unwrap();
        assert!(csp.contains("form-action 'self' https://idp.example.com/sso"));
    }

    #[test]
    fn url_that_is_not_header_safe_is_unrepresentable() {
        // A control character cannot be represented in a CSP header value. It is
        // rejected when the URL is parsed, so the page is never served with a
        // degraded CSP; `autosubmit_post_response` keeps its `HeaderValue` check
        // as a backstop.
        assert!(EndpointUrl::from_metadata("https://idp\u{0000}.example", "SSO").is_err());
    }
}
