pub(crate) mod crypto;
#[cfg(feature = "database")]
pub(crate) mod database;

pub(crate) mod persistence;

mod event;
mod filesystem;
mod health;
pub(crate) mod memory;
mod registry;
mod store_handle;
mod stream_id;

pub(crate) use event::EncryptedEvent;
pub use event::{Event, EventHash, GENESIS_HASH, StoreEvent};
pub use health::{DbHealth, run_db_prober};
pub use persistence::StorePersistence;
pub use registry::StoreRegistry;
pub use store_handle::Store;
#[cfg(test)]
pub(crate) use store_handle::StoreBackend;
pub use stream_id::StreamId;

pub(crate) use event::{chain_hash, event_aad};

use chrono::{DateTime, Utc};
use serde::de::DeserializeOwned;

use crate::{AppError, ElectionConfig, Scope, crypto::EventCipher};

/// Decryption-free metadata about a persisted stream, read from the backend's
/// index without replaying (or warming) it. The political group name is absent:
/// it lives in the encrypted payloads.
#[derive(Clone, Debug)]
pub struct StreamMeta {
    pub stream_id: StreamId,
    pub election: ElectionConfig,
    /// Number of events, i.e. the last event id (events are appended `1..=n`).
    pub event_count: usize,
    pub created_at: Option<DateTime<Utc>>,
    pub last_event_at: Option<DateTime<Utc>>,
}

pub trait StoreData: Default + Send + Sync + 'static {
    type Event: Event;

    /// Apply a fully wrapped store event to the data projection.
    fn apply(&mut self, event: StoreEvent<Self::Event>);

    /// All events applied to this projection, in order.
    fn events(&self) -> &[StoreEvent<Self::Event>];

    /// Return the last applied event ID for this data instance.
    fn last_event_id(&self) -> usize {
        self.events().last().map(|e| e.event_id).unwrap_or(0)
    }

    /// Return the chain hash of the last applied event, or [`GENESIS_HASH`] if
    /// no events have been applied yet.
    fn last_event_hash(&self) -> EventHash {
        self.events().last().map(|e| e.hash).unwrap_or(GENESIS_HASH)
    }

    fn scope() -> Scope;
}

/// Decrypt persisted events and apply the ones `data` has not seen yet.
///
/// `events` yields the stored events in ascending event order; events at or
/// below the projection's last ID are skipped. Shared by the filesystem and
/// database backends.
pub(crate) fn apply_encrypted_events<D>(
    data: &mut D,
    cipher: &EventCipher,
    events: impl IntoIterator<Item = EncryptedEvent>,
) -> Result<(), AppError>
where
    D: StoreData,
    D::Event: DeserializeOwned,
{
    let mut prev_hash = data.last_event_hash();

    for EncryptedEvent {
        event_id,
        created_at,
        hash,
        payload: encrypted_payload,
    } in events
    {
        if data.last_event_id() >= event_id {
            continue;
        }

        // Verify the chain over the stored blob before touching the plaintext.
        // Gated behind a feature flag: it costs a SHA-256 over every loaded
        // event. (Reordering, removal, and in-place edits are still caught by
        // the AES-GCM tag, since `prev_hash` is part of the associated data.)
        #[cfg(feature = "verify-event-hash-chain")]
        if chain_hash(&prev_hash, event_id, created_at, &encrypted_payload) != hash {
            return Err(AppError::EventDecodeError(format!(
                "hash chain broken at event {event_id}"
            )));
        }

        let aad = event_aad(event_id, created_at, &prev_hash);
        let payload = cipher.decrypt::<D::Event>(encrypted_payload, &aad)?;
        prev_hash = hash;
        data.apply(StoreEvent {
            event_id,
            payload,
            created_at,
            hash,
        });
    }

    Ok(())
}
