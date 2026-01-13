use chrono::{DateTime, Duration, Utc};
use rand::{Rng, distr::Alphanumeric};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    fmt::Display,
    sync::{Arc, RwLock},
};
use tracing::info;

/// Number of minutes a CSRF token remains valid.
pub const CSRF_TOKEN_TTL_MINUTES: i64 = 30;

#[derive(Clone, Debug, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub struct TokenValue(pub String);

impl Display for TokenValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

#[derive(Clone, Debug)]
pub struct CsrfToken {
    pub value: TokenValue,
    pub expires_at: DateTime<Utc>,
}

#[derive(Default, Clone)]
pub struct CsrfTokens {
    tokens: Arc<RwLock<HashMap<TokenValue, DateTime<Utc>>>>,
}

fn random_token_value() -> TokenValue {
    TokenValue(
        rand::rng()
            .sample_iter(&Alphanumeric)
            .take(24)
            .map(char::from)
            .collect(),
    )
}

impl CsrfTokens {
    pub fn issue(&self) -> CsrfToken {
        let expires_at = Utc::now() + Duration::minutes(CSRF_TOKEN_TTL_MINUTES);
        let token = CsrfToken {
            value: random_token_value(),
            expires_at,
        };

        let mut tokens = self.tokens.write().expect("csrf token store poisoned");
        Self::purge_locked(&mut tokens);
        tokens.insert(token.value.clone(), expires_at);

        info!(
            "issued new CSRF token {} expiring at {expires_at}",
            token.value.0
        );

        token
    }

    pub fn consume(&self, value: &TokenValue) -> bool {
        let mut tokens = self.tokens.write().expect("csrf token store poisoned");
        Self::purge_locked(&mut tokens);
        info!("verifying CSRF token {}", value.0);

        tokens.remove(value).is_some()
    }

    fn purge_locked(tokens: &mut HashMap<TokenValue, DateTime<Utc>>) {
        let now = Utc::now();
        tokens.retain(|_, expires_at| *expires_at > now);
    }
}

pub trait WithCsrfToken: Default {
    fn with_csrf_token(self, csrf_token: CsrfToken) -> Self;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn issuing_token_stores_and_marks_active() {
        let tokens = CsrfTokens::default();

        let token = tokens.issue();

        assert!(tokens.consume(&token.value));
        assert!(!tokens.consume(&token.value));
    }
}
