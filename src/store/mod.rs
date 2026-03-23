#[cfg(feature = "database")]
pub(crate) mod database;

mod event;
mod filesystem;
mod persistence;
mod registry;
mod store_handle;

pub use event::StoreEvent;
pub use persistence::StorePersistence;
pub use registry::StoreRegistry;
pub use store_handle::Store;

pub trait StoreData: Default + Send + Sync + 'static {
    type Event;

    /// Apply a fully wrapped store event to the data projection.
    fn apply(&mut self, event: StoreEvent<Self::Event>);
    /// Return the last applied event ID for this data instance.
    fn last_event_id(&self) -> usize;
    /// Update the last applied event ID for this data instance.
    fn set_last_event_id(&mut self, event_id: usize);
}
