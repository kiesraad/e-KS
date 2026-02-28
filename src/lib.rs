/// Application specific modules
mod app;

/// Generic modules
mod core;
mod error;
mod form;
mod pagination;
mod state;
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
    Config, ElectionConfig, ElectoralDistrict, HtmlTemplate, Locale, constants, get_env, logging,
    server, translate,
};
pub use error::{AppError, AppResponse, ErrorResponse, render_error_pages};
pub use form::{CsrfToken, CsrfTokens, Form, TokenValue};
pub use state::AppState;
pub use utils::{OptionStringExt, QueryParamState, new_type, redirect_success};

#[cfg(test)]
pub use utils::test_utils;

pub type AppStore = store::Store<AppStoreData>;
