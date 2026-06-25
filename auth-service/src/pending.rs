//! In-memory store of outstanding AuthnRequest IDs for the `InResponseTo`
//! replay check (eID §7.6.3.5 rule 4 / §9.7). Process-local: multi-instance
//! deployments should back this with shared storage (e.g. the database).

use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
    time::{SystemTime, UNIX_EPOCH},
};

/// Retention window for outstanding AuthnRequest IDs (eID §9.7). Matches the
/// 15-minute artifact validity (eID §7.1/§7.5); expired entries are swept.
pub const PENDING_REQUEST_TTL: std::time::Duration = std::time::Duration::from_secs(15 * 60);

const PENDING_REQUEST_TTL_MS: u64 = PENDING_REQUEST_TTL.as_secs() * 1000;

#[derive(Clone, Default)]
pub struct PendingRequests {
    inner: Arc<Mutex<HashMap<String, u64>>>,
}

impl PendingRequests {
    /// Record an outgoing AuthnRequest ID, sweeping expired entries first.
    pub fn register(&self, id: String) {
        let now = unix_ms();
        let mut pending = self.inner.lock().unwrap();
        pending.retain(|_, &mut created| now.saturating_sub(created) < PENDING_REQUEST_TTL_MS);
        pending.insert(id, now);
    }

    /// Consume a pending request ID (eID §7.6.3.5 rule 4 / §9.7), sweeping expired
    /// entries first. Returns `false` for unknown, expired, or already-consumed IDs.
    pub fn consume_if_pending(&self, id: &str) -> bool {
        let now = unix_ms();
        let mut pending = self.inner.lock().unwrap();
        pending.retain(|_, &mut created| now.saturating_sub(created) < PENDING_REQUEST_TTL_MS);
        pending.remove(id).is_some()
    }
}

fn unix_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn consume_if_pending_matches_once_then_rejects_replay() {
        let pending = PendingRequests::default();
        pending.register("abc".into());
        pending.register("def".into());

        assert!(pending.consume_if_pending("abc"));
        assert!(!pending.consume_if_pending("abc")); // replay rejected
        assert!(pending.consume_if_pending("def")); // other IDs are unaffected
    }

    #[test]
    fn consume_if_pending_rejects_unknown_id() {
        let pending = PendingRequests::default();
        assert!(!pending.consume_if_pending("never-registered"));
    }

    #[test]
    fn expired_pending_requests_are_rejected_and_swept() {
        let pending = PendingRequests::default();
        pending
            .inner
            .lock()
            .unwrap()
            .insert("stale".into(), unix_ms() - PENDING_REQUEST_TTL_MS - 1);

        assert!(!pending.consume_if_pending("stale")); // expired -> rejected (eID §9.7)
        assert!(pending.inner.lock().unwrap().is_empty()); // swept in same call
    }
}
