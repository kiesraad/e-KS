/// Application specific modules
mod core;

/// Generic modules
mod common;
mod error;
pub mod filters;
mod form;
mod pages;
mod pagination;
pub mod router;
mod store;
mod structs;
mod submit;

#[cfg(feature = "fixtures")]
pub mod fixtures;

pub use common::{
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
pub use core::{
    authorised_agents, candidate_lists, candidates, list_submitters, persons, political_groups,
    substitute_list_submitters,
};
pub use error::{AppError, AppResponse, ErrorResponse, render_error_pages};
pub use form::{CsrfToken, CsrfTokens, Form, TokenValue};
pub use store::{AppEvent, AppStore, AppStoreData};
pub use structs::{
    Bsn, CountryCode, Date, DutchAddress, DutchAddressForm, FirstName, FullName, FullNameForm,
    HouseNumber, HouseNumberAddition, Initials, LastName, LastNamePrefix, LegalName, Locality,
    PlaceOfResidence, PostalCode, StreetName, UtcDateTime,
};

#[cfg(test)]
pub use common::test_utils;
