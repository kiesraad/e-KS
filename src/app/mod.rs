pub mod audit_log;
pub mod candidate_lists;
pub mod candidates;
pub mod common;
pub mod finalise;
pub mod list_designation;
pub mod list_submitters;
pub mod name_authorisations;
pub mod persons;
pub mod political_groups;
pub mod substitute_list_submitters;

mod context;
mod error_response;
mod middleware;
mod store;

pub use context::Context;
pub use error_response::render_error_pages;
pub use middleware::{
    eks_key::eks_key_middleware,
    health::health_router,
    maintenance::db_gate_middleware,
    session::{csb_store_middleware, session_middleware, store_middleware},
};
pub use store::{AppEvent, AppStoreData};

#[cfg(feature = "dev-features")]
pub use middleware::dev_login;

pub(crate) use store::request_extractor;

#[cfg(not(feature = "memory-serve"))]
pub use middleware::proxy::proxy_handler;

#[cfg(test)]
mod error_response_tests;
