#[cfg(feature = "database")]
pub(crate) mod database;

pub(crate) mod encryption;
pub(crate) mod persistence;

mod event;
mod filesystem;
mod health;
pub(crate) mod memory;
mod registry;
mod store_handle;

pub use encryption::EventEncryption;
pub use event::{Event, GENESIS_HASH, StoreEvent};
pub use health::{DbHealth, run_db_prober};
pub use persistence::{StorePersistence, StreamMeta};
pub use registry::StoreRegistry;
pub use store_handle::Store;

pub(crate) use event::{chain_hash, event_aad};

use chrono::{DateTime, Utc};
use serde::de::DeserializeOwned;

use crate::{AppError, Scope};
use encryption::EventCipher;

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
    fn last_event_hash(&self) -> [u8; 32] {
        self.events().last().map(|e| e.hash).unwrap_or(GENESIS_HASH)
    }

    fn scope() -> Scope;
}

/// Decrypt persisted events and apply the ones `data` has not seen yet.
///
/// `events` yields `(event_id, created_at, hash, encrypted_payload)` in
/// ascending event order; events at or below the projection's last ID are
/// skipped. Shared by the filesystem and database backends.
pub(crate) fn apply_encrypted_events<D>(
    data: &mut D,
    cipher: &EventCipher,
    events: impl IntoIterator<Item = (usize, DateTime<Utc>, [u8; 32], Vec<u8>)>,
) -> Result<(), AppError>
where
    D: StoreData,
    D::Event: DeserializeOwned,
{
    let mut prev_hash = data.last_event_hash();

    for (event_id, created_at, hash, encrypted_payload) in events {
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
