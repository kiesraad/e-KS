//! In-memory store of outstanding AuthnRequest IDs for the `InResponseTo`
//! replay check (eID §7.6.3.5 rule 4 / §9.7).
//!
//! This is the default, process-local implementation an embedding application
//! can use for [`AuthState::consume_if_pending`](crate::AuthState) and
//! [`register_pending_request`](crate::AuthState::register_pending_request)
//! when it does not need the IDs shared across instances. Deployments that run
//! more than one application server should back the storage with something
//! shared (e.g. the database) instead, because a login started on one server
//! may have its ACS callback handled by another.

use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
    time::{SystemTime, UNIX_EPOCH},
};

/// How long an outstanding AuthnRequest ID is retained for the `InResponseTo`
/// replay check (eID §9.7). The artifact it correlates to is valid for at most
/// 15 minutes (eID §7.1/§7.5), so a legitimate flow always completes within
/// this window. Entries older than this are swept, which bounds memory and the
/// replay-acceptance window for abandoned flows (cancellations/errors leave no
/// assertion to correlate, so they can only be reclaimed by age).
pub const PENDING_REQUEST_TTL: std::time::Duration = std::time::Duration::from_secs(15 * 60);

const PENDING_REQUEST_TTL_MS: u64 = PENDING_REQUEST_TTL.as_secs() * 1000;

#[derive(Clone, Default)]
pub struct PendingRequests {
    inner: Arc<Mutex<HashMap<String, u64>>>,
}

impl PendingRequests {
    /// Record an outgoing AuthnRequest ID, sweeping any entries that have
    /// outlived [`PENDING_REQUEST_TTL`] first so abandoned flows cannot
    /// accumulate.
    pub fn register(&self, id: String) {
        let now = unix_ms();
        let mut pending = self.inner.lock().unwrap();
        pending.retain(|_, &mut created| now.saturating_sub(created) < PENDING_REQUEST_TTL_MS);
        pending.insert(id, now);
    }

    /// Atomically check whether `id` is a still-valid outstanding request and,
    /// if so, consume it so it can never be matched again (eID §7.6.3.5 rule 4 /
    /// §9.7). Returns `false` for an unknown, expired, or already-consumed ID.
    ///
    /// Expired entries are swept first, so an ID older than
    /// [`PENDING_REQUEST_TTL`] is gone before the lookup and therefore never
    /// matches, and abandoned flows cannot accumulate.
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

        // A registered ID matches exactly once...
        assert!(pending.consume_if_pending("abc"));
        // ...and a replay of the same ID is rejected.
        assert!(!pending.consume_if_pending("abc"));
        // Other registered IDs are unaffected.
        assert!(pending.consume_if_pending("def"));
    }

    #[test]
    fn consume_if_pending_rejects_unknown_id() {
        let pending = PendingRequests::default();
        assert!(!pending.consume_if_pending("never-registered"));
    }

    #[test]
    fn expired_pending_requests_are_rejected_and_swept() {
        let pending = PendingRequests::default();
        // Inject a stale entry whose timestamp is well past the TTL.
        pending
            .inner
            .lock()
            .unwrap()
            .insert("stale".into(), unix_ms() - PENDING_REQUEST_TTL_MS - 1);

        // An expired ID no longer matches (so a stale InResponseTo is rejected,
        // eID §9.7)...
        assert!(!pending.consume_if_pending("stale"));
        // ...and the stale entry has been physically swept in the same call.
        assert!(pending.inner.lock().unwrap().is_empty());
    }
}
