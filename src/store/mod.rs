#[cfg(feature = "database")]
pub(crate) mod database;

pub(crate) mod encryption;
mod event;
mod filesystem;
mod persistence;
mod registry;
mod store_handle;

pub use encryption::EventEncryption;
pub use event::{GENESIS_HASH, StoreEvent};
pub use persistence::StorePersistence;
pub use registry::StoreRegistry;
pub use store_handle::Store;

pub(crate) use event::{chain_hash, event_aad};

pub trait StoreData: Default + Send + Sync + 'static {
    type Event;

    /// Apply a fully wrapped store event to the data projection.
    fn apply(&mut self, event: StoreEvent<Self::Event>);
    /// Return the last applied event ID for this data instance.
    fn last_event_id(&self) -> usize;
    /// Return the chain hash of the last applied event, or [`GENESIS_HASH`] if
    /// no events have been applied yet.
    fn last_event_hash(&self) -> [u8; 32];
}
