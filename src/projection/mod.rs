//! Event-sourced stream projections and the events that drive them.
mod csb_data;
mod csb_main;
mod csb_store;
mod pg_data;
mod pg_store;
mod request_state;

pub use csb_data::{CsbAction, CsbStoreData, WithCorrections};
pub use csb_main::{CSB_MAIN_STREAM_ID, CsbMainAction, CsbMainStoreData};
pub use csb_store::CsbStore;
pub use pg_data::{PgEvent, PgStoreData};
pub use pg_store::PgStore;
pub use request_state::AppRequestState;
#[cfg(test)]
pub use {csb_data::CsbEvent, csb_main::CsbMainEvent};

pub type CsbStream = crate::store::Store<CsbStoreData>;
pub type CsbMainStore = crate::store::Store<CsbMainStoreData>;
