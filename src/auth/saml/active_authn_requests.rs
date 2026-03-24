use chrono::{DateTime, Duration, Utc};
use parking_lot::RwLock;
use std::{collections::HashMap, sync::Arc};

/// Number of minutes a CSRF token remains valid.
pub const AUTHN_REQUEST_TTL_MINUTES: i64 = 30;

#[derive(Default, Clone)]
pub struct ActiveAuthnRequests {
    ids: Arc<RwLock<HashMap<String, DateTime<Utc>>>>,
}

impl ActiveAuthnRequests {
    pub fn add(&self, id: String) {
        tracing::debug!("Adding active AuthnRequest with ID: {}", id);
        let expires_at = Utc::now() + Duration::minutes(AUTHN_REQUEST_TTL_MINUTES);

        let mut ids = self.ids.write();
        Self::purge_locked(&mut ids);
        ids.insert(id, expires_at);
    }

    pub fn remove(&self, id: &str) {
        tracing::debug!("Removing active AuthnRequest with ID: {}", id);
        let mut ids = self.ids.write();
        Self::purge_locked(&mut ids);
        ids.remove(id);
    }

    pub fn list_all(&self) -> Vec<String> {
        let mut ids = self.ids.write();
        Self::purge_locked(&mut ids);
        ids.keys().cloned().collect()
    }

    fn purge_locked(ids: &mut HashMap<String, DateTime<Utc>>) {
        let now = Utc::now();
        ids.retain(|_, &mut expires_at| expires_at > now);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_active_authn_requests() {
        let active_requests = ActiveAuthnRequests::default();

        active_requests.add("request1".to_string());
        active_requests.add("request2".to_string());

        let ids = active_requests.list_all();
        assert_eq!(ids.len(), 2);
        assert!(ids.contains(&"request1".to_string()));
        assert!(ids.contains(&"request2".to_string()));

        active_requests.remove("request2");
        let ids = active_requests.list_all();
        assert_eq!(ids.len(), 1);
        assert!(ids.contains(&"request1".to_string()));
    }

    #[test]
    fn test_active_authn_requests_expiry() {
        let active_requests = ActiveAuthnRequests::default();

        {
            let mut ids = active_requests.ids.write();
            ids.insert("request1".to_string(), Utc::now() - Duration::minutes(1));
            ids.insert("request2".to_string(), Utc::now() + Duration::minutes(1));
        }

        // should not list expired requests
        let ids = active_requests.list_all();
        assert_eq!(ids.len(), 1);
        assert!(ids.contains(&"request2".to_string()));

        // should have purged expired requests
        let ids = active_requests.ids.read();
        assert!(!ids.contains_key("request1"));
        assert!(ids.contains_key("request2"));
    }
}
