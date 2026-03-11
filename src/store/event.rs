//! Event wrapper used by the event-sourced store.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoreEvent<E> {
    /// Monotonic event identifier within a stream.
    pub event_id: usize,
    /// Domain-specific event payload.
    pub payload: E,
    /// Timestamp recorded when the event was created.
    pub created_at: DateTime<Utc>,
}

impl<E> StoreEvent<E> {
    /// Construct a new store event with the given ID and payload.
    /// `created_at` is set to the current UTC time.
    pub fn new(event_id: usize, payload: E) -> Self {
        Self {
            event_id,
            payload,
            created_at: Utc::now(),
        }
    }

    pub fn new_at(event_id: usize, payload: E, created_at: DateTime<Utc>) -> Self {
        Self {
            event_id,
            payload,
            created_at,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_sets_timestamp_and_fields() {
        let before = Utc::now();
        let event = StoreEvent::new(3, "payload");
        let after = Utc::now();

        assert_eq!(event.event_id, 3);
        assert_eq!(event.payload, "payload");
        assert!(event.created_at >= before);
        assert!(event.created_at <= after);
    }

    #[test]
    fn new_at_uses_provided_timestamp() {
        let timestamp = DateTime::from_timestamp(1_700_000_000, 0).unwrap();
        let event = StoreEvent::new_at(7, 42, timestamp);

        assert_eq!(event.event_id, 7);
        assert_eq!(event.payload, 42);
        assert_eq!(event.created_at, timestamp);
    }
}
