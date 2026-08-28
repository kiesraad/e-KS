//! Event-sourced stream projections and the events that drive them.
mod csb_data;
mod csb_main;
mod pg_data;
mod pg_store;
mod request_state;

pub use csb_data::{CsbEvent, CsbStoreData, WithCorrections};
#[cfg(any(test, feature = "dev-features"))]
pub use csb_main::CsbMainEvent;
pub use csb_main::{CSB_MAIN_STREAM_ID, CsbMainStoreData};
pub use pg_data::{PgEvent, PgStoreData};
pub use pg_store::PgStore;
pub use request_state::AppRequestState;

pub type CsbStore = crate::store::Store<CsbStoreData>;
pub type CsbMainStore = crate::store::Store<CsbMainStoreData>;
