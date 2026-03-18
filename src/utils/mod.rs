//! Utilities and small helpers shared across the application.
pub mod no_cache_headers;
mod option_string_ext;
mod query_param_state;
mod redirect;
mod slugify_teletex;

pub mod id_newtype;
pub mod transparent_string;

#[cfg(feature = "livereload")]
pub mod livereload;

#[cfg(any(feature = "dev-features", not(feature = "memory-serve")))]
pub mod proxy;

#[cfg(feature = "embed-typst")]
pub mod embed_typst;

#[cfg(test)]
pub mod test_utils;

pub use no_cache_headers::generate_attachment_headers;
pub use option_string_ext::{OptionAsStrExt, OptionStringExt};
pub use query_param_state::QueryParamState;
pub use redirect::redirect_success;
pub use slugify_teletex::slugify_teletex;
