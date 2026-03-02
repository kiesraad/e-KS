/// Application specific modules
mod app;

/// Generic modules
mod core;
mod error;
mod form;
mod pagination;
mod store;

pub mod filters;
pub mod router;
pub mod utils;

#[cfg(feature = "fixtures")]
pub mod fixtures;

pub use app::{
    AppEvent, AppStoreData, Context, authorised_agents, candidate_lists, candidates, common,
    list_submitters, persons, political_groups, submit, substitute_list_submitters,
};
pub use core::{
    AppState, Config, ElectionConfig, ElectoralDistrict, HtmlTemplate, Locale, OptionStringExt,
    QueryParamState, constants, get_env, logging, new_type, redirect_success, server, translate,
};
pub use error::{AppError, AppResponse, ErrorResponse, render_error_pages};
pub use form::{CsrfToken, CsrfTokens, Form, TokenValue};

#[cfg(test)]
pub use core::test_utils;

pub type Store = store::Store<AppStoreData>;
