//! Utilities and small helpers shared across the application.
mod option_string_ext;
mod query_param_state;
mod redirect;

pub mod new_type;

#[cfg(feature = "livereload")]
pub mod livereload;

#[cfg(any(feature = "dev-features", not(feature = "memory-serve")))]
pub mod proxy;

#[cfg(feature = "embed-typst")]
pub mod embed_typst;

#[cfg(test)]
pub mod test_utils;

pub use option_string_ext::OptionStringExt;

pub use query_param_state::QueryParamState;
pub use redirect::redirect_success;
