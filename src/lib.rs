/// Generic modules
mod common;
mod error;
mod form;
mod pages;
mod pagination;

/// Application specific modules
mod candidate_lists;
mod persons;

#[cfg(feature = "fixtures")]
pub mod fixtures;

pub use common::{
    config::Config,
    constants,
    context::Context,
    election::{ElectionConfig, ElectoralDistrict},
    filters, locale,
    locale::Locale,
    logging, router, server,
    state::{AppState, DbConnection},
    templates::HtmlTemplate,
    translate,
};
pub use error::{AppError, AppResponse, ErrorResponse};
pub use form::{CsrfToken, CsrfTokens};

#[cfg(test)]
pub use common::test_utils;
