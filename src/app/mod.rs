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

pub mod eks_key;
pub mod health;

#[cfg(any(
    all(feature = "dev-features", not(feature = "embed-bag")),
    not(feature = "memory-serve")
))]
pub mod proxy;

mod context;
mod error_response;
mod event;
mod extractor;
mod getters;
mod store;
mod store_extractor;

pub use context::Context;
pub use error_response::{ErrorResponse, render_error_pages};
pub use event::AppEvent;
pub use store::AppStoreData;

#[cfg(test)]
mod store_tests;

#[cfg(test)]
mod health_tests;

#[cfg(test)]
mod error_response_tests;
