use chrono::{DateTime, Duration, Utc};
use rand::{Rng, distr::Alphanumeric};
use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};
use tracing::info;

/// Number of minutes a CSRF token remains valid.
pub const CSRF_TOKEN_TTL_MINUTES: i64 = 30;

#[derive(Clone, Debug)]
pub struct CsrfToken {
    pub value: String,
    pub expires_at: DateTime<Utc>,
}

#[derive(Default, Clone)]
pub struct CsrfTokens {
    tokens: Arc<RwLock<HashMap<String, DateTime<Utc>>>>,
}

fn random_string() -> String {
    rand::rng()
        .sample_iter(&Alphanumeric)
        .take(24)
        .map(char::from)
        .collect()
}

impl CsrfTokens {
    pub fn issue(&self) -> CsrfToken {
        let expires_at = Utc::now() + Duration::minutes(CSRF_TOKEN_TTL_MINUTES);
        let token = CsrfToken {
            value: random_string(),
            expires_at,
        };

        let mut tokens = self.tokens.write().expect("csrf token store poisoned");
        Self::purge_locked(&mut tokens);
        tokens.insert(token.value.clone(), expires_at);

        info!(
            "issued new CSRF token {} expiring at {expires_at}",
            token.value
        );

        token
    }

    pub fn consume(&self, value: &str) -> bool {
        let mut tokens = self.tokens.write().expect("csrf token store poisoned");
        Self::purge_locked(&mut tokens);

        info!("verifying CSRF token {}", value);

        tokens.remove(value).is_some()
    }

    fn purge_locked(tokens: &mut HashMap<String, DateTime<Utc>>) {
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
