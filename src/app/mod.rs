pub mod audit_log;
pub mod authorised_agents;
pub mod candidate_lists;
pub mod candidates;
pub mod common;
pub mod list_submitters;
pub mod persons;
pub mod political_groups;
pub mod submit;
pub mod substitute_list_submitters;

mod context;
mod error_response;
mod event;
mod middleware;
mod store;

pub use context::Context;
pub use error_response::{ErrorResponse, render_error_pages};
pub use event::AppEvent;
pub use middleware::{eks_key::eks_key_middleware, health::health_router};
pub use store::AppStoreData;

pub(crate) use store::request_extractor;

#[cfg(any(
    all(feature = "dev-features", not(feature = "embed-bag")),
    not(feature = "memory-serve")
))]
pub use middleware::proxy::proxy_handler;

#[cfg(test)]
mod error_response_tests;
