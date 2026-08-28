//! GitHub OAuth login for CSB (central electoral committee) users.
//!
//! Fully separate from the political-group login (SAML DigiD/TVS, implemented
//! in the auth-service): committee members authenticate with their GitHub
//! account through the OAuth authorization-code flow and must appear on the
//! configured allowlist of numeric GitHub account ids (see
//! [`crate::GithubOauthConfig`]).
//!
//! Flow and defences:
//! - `POST /csb/login` registers a one-shot random `state` nonce in the
//!   pending-request store (15-minute TTL, single use) and binds it to the
//!   browser with a short-lived cookie, then redirects to GitHub.
//! - `GET /csb/login/callback` accepts the nonce only when it matches the
//!   browser's cookie (constant-time) and is still pending; the code is then
//!   exchanged server-side and the account id checked against the allowlist.
//! - A successful login drops any pre-existing session (fixation defence) and
//!   creates a session scoped to [`crate::Scope::CentralElectoralCommittee`],
//!   recording the login on the shared CSB main stream for the audit log.

mod github;
mod pages;
mod paths;
mod state_cookie;

pub use pages::public_router;
pub use paths::{CsbLoginCallbackPath, CsbLoginPath};

use crate::{AppError, Config, GithubOauthConfig};

/// The GitHub OAuth config, or 404 when this deployment has no CSB login.
fn require_github_oauth(config: &Config) -> Result<&GithubOauthConfig, AppError> {
    config
        .github_oauth
        .as_ref()
        .ok_or(AppError::GenericNotFound)
}

/// Pending-request id for an OAuth `state` nonce. Namespaced so nonces can
/// never collide with SAML AuthnRequest ids in the shared store.
fn pending_state_id(nonce: &str) -> String {
    format!("github-oauth:{nonce}")
}

#[cfg(test)]
pub(crate) mod test_support {
    use secrecy::SecretString;

    use crate::{Config, GithubOauthConfig, GithubUserId};

    /// The GitHub account id on the test allowlist.
    pub(crate) fn allowed_user_id() -> GithubUserId {
        "583231".parse().expect("valid id")
    }

    /// Test config with the GitHub OAuth login enabled.
    pub(crate) fn github_test_config() -> Config {
        let mut config = Config::new_test();
        config.github_oauth = Some(GithubOauthConfig {
            client_id: "Iv1.testclient".to_string(),
            client_secret: SecretString::from("test-secret"),
            allowed_user_ids: vec![allowed_user_id()],
        });
        config
    }
}
