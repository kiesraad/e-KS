//! Utilities and small helpers shared across the application.
mod bsn;
mod format_hash;
mod option_string_ext;
mod query_param_state;
mod redirect;
mod slugify_teletex;

pub mod id_newtype;
pub mod locality_aliases;
pub mod no_cache_headers;
pub mod transparent_string;

#[cfg(feature = "livereload")]
pub mod livereload;

#[cfg(feature = "embed-typst")]
pub mod embed_typst;

#[cfg(feature = "embed-bag")]
pub mod embed_bag;

#[cfg(test)]
pub mod test_utils;

pub use bsn::random_bsn;
pub use format_hash::format_hash;
pub use no_cache_headers::generate_attachment_headers;
pub use option_string_ext::{OptionAsStrExt, OptionStringExt};
pub use query_param_state::QueryParamState;
pub use redirect::redirect_success;
pub use slugify_teletex::slugify_teletex;
