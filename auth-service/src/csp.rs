//! The Content-Security-Policy this crate sets on its own responses.
//!
//! Only the SAML auto-submit page needs one: it POSTs to the IdP endpoint from
//! the verified RD metadata, which a strict `form-action 'self'` blocks. Every
//! other response is left without the header, so the embedding application's
//! policy applies.
//!
//! A CSP header does not merge with another, so the widened policy has to
//! restate the whole lockdown. `strict()` is public so an application can build
//! its global policy from this same list rather than keep a second copy.

use std::borrow::Cow;

/// The strict directives, in emission order. Mirrors what the embedding
/// application applies elsewhere. Directives `default-src 'none'` already
/// covers are spelled out, so a change to a browser's fallback chain cannot
/// widen the policy.
///
/// SECURITY (ordering): CSP is first-occurrence-wins per directive, and
/// [`ContentSecurityPolicy::widen`] keeps a directive in its original position.
/// `script-src` is therefore listed *before* `form-action`, the only directive
/// that ever carries an externally-derived value: a second `script-src`
/// smuggled into that value could not displace ours.
const DEFAULT: &[(&str, &str)] = &[
    ("default-src", "'none'"),
    ("base-uri", "'none'"),
    ("script-src", "'self'"),
    ("style-src", "'self'"),
    ("img-src", "'self'"),
    ("font-src", "'self'"),
    ("connect-src", "'self'"),
    ("object-src", "'none'"),
    ("media-src", "'none'"),
    ("frame-src", "'none'"),
    ("child-src", "'none'"),
    ("worker-src", "'none'"),
    ("manifest-src", "'none'"),
    ("form-action", "'self'"),
    ("frame-ancestors", "'none'"),
    ("require-trusted-types-for", "'script'"),
    ("trusted-types", "'none'"),
];

/// A Content-Security-Policy: [`ContentSecurityPolicy::strict`] plus any
/// per-response widening. Render with `to_string`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContentSecurityPolicy {
    directives: Vec<(&'static str, Cow<'static, str>)>,
}

impl ContentSecurityPolicy {
    /// The strict base every widening starts from.
    pub fn strict() -> Self {
        Self {
            directives: DEFAULT
                .iter()
                .map(|(name, value)| (*name, Cow::Borrowed(*value)))
                .collect(),
        }
    }

    /// Append `source` to `directive`, keeping its position in `DEFAULT` (see
    /// the ordering note there). A directive not in the default is appended.
    pub fn widen(mut self, directive: &'static str, source: &str) -> Self {
        match self.directives.iter_mut().find(|(n, _)| *n == directive) {
            Some((_, value)) => *value = Cow::Owned(format!("{value} {source}")),
            None => self
                .directives
                .push((directive, Cow::Owned(source.to_owned()))),
        }
        self
    }
}

impl std::fmt::Display for ContentSecurityPolicy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (i, (name, value)) in self.directives.iter().enumerate() {
            if i > 0 {
                f.write_str("; ")?;
            }
            write!(f, "{name} {value}")?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strict_policy_denies_by_default() {
        let csp = ContentSecurityPolicy::strict().to_string();
        for directive in [
            "default-src 'none'",
            "base-uri 'none'",
            "form-action 'self'",
            "frame-ancestors 'none'",
            "object-src 'none'",
            "require-trusted-types-for 'script'",
            "trusted-types 'none'",
        ] {
            assert!(
                csp.contains(directive),
                "CSP must keep `{directive}`: {csp}"
            );
        }
        assert!(!csp.contains("unsafe-inline") && !csp.contains("unsafe-eval"));
    }

    #[test]
    fn widen_appends_to_the_directive_in_place() {
        let csp = ContentSecurityPolicy::strict()
            .widen("form-action", "https://idp.example.com/sso")
            .to_string();
        assert!(csp.contains("form-action 'self' https://idp.example.com/sso"));
        // Widening one directive leaves the rest of the lockdown intact.
        assert!(csp.contains("default-src 'none'"));
        assert!(csp.contains("script-src 'self'"));
        // And keeps the position: script-src still wins over anything smuggled
        // into the form-action value.
        assert!(csp.find("script-src 'self'") < csp.find("form-action"));
    }

    #[test]
    fn widen_adds_a_directive_the_default_lacks() {
        let csp = ContentSecurityPolicy::strict()
            .widen("report-to", "csp-endpoint")
            .to_string();
        assert!(csp.ends_with("report-to csp-endpoint"));
    }
}
