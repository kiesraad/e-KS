/// Application specific modules
mod app;

/// Generic modules
mod core;
mod error;
pub mod filters;
mod form;
mod pagination;
pub mod router;
mod store;

#[cfg(feature = "fixtures")]
pub mod fixtures;

pub use app::{
    authorised_agents, candidate_lists, candidates, common, list_submitters, persons,
    political_groups, substitute_list_submitters, submit
};
pub use core::{
    config::Config,
    constants,
    context::Context,
    election::{ElectionConfig, ElectoralDistrict},
    locale,
    locale::Locale,
    logging, new_type,
    option_string_ext::OptionStringExt,
    query_param_state::QueryParamState,
    redirect::redirect_success,
    server,
    state::AppState,
    templates::HtmlTemplate,
    translate,
};
pub use error::{AppError, AppResponse, ErrorResponse, render_error_pages};
pub use form::{CsrfToken, CsrfTokens, Form, TokenValue};
pub use store::{AppEvent, AppStore, AppStoreData};

#[cfg(test)]
pub use core::test_utils;
