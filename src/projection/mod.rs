//! Event-sourced stream projections and the events that drive them.
mod csb_data;
mod csb_main;
mod pg_data;
mod pg_store;
mod request_state;

#[cfg(test)]
pub use csb_data::CsbEvent;
pub use csb_data::{CsbAction, CsbStoreData, WithCorrections};
#[cfg(test)]
pub use csb_main::CsbMainEvent;
pub use csb_main::{CSB_MAIN_STREAM_ID, CsbMainAction, CsbMainStoreData};
pub use pg_data::{PgEvent, PgStoreData};
pub use pg_store::PgStore;
pub use request_state::AppRequestState;

pub type CsbStore = crate::store::Store<CsbStoreData>;
pub type CsbMainStore = crate::store::Store<CsbMainStoreData>;
