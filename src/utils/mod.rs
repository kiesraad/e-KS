//! Utilities and small helpers shared across the application.
mod abbreviate_str;
#[cfg(feature = "dev-features")]
mod bsn;
mod format_hash;
mod option_string_ext;
mod overlay;
mod query_param_state;
mod query_suffix;
mod redirect;
mod request_flags;
mod sha256_hex;
mod slugify_teletex;
mod storage_url;

pub mod bag;
pub mod id_newtype;
pub mod locality_aliases;
pub mod no_cache_headers;
pub mod transparent_string;

#[cfg(feature = "livereload")]
pub mod livereload;

#[cfg(test)]
pub mod test_utils;

pub use abbreviate_str::abbreviate_str;
#[cfg(feature = "dev-features")]
pub use bsn::random_bsn;
pub use format_hash::{format_hash, parse_hash_prefix};
pub use option_string_ext::{OptionAsStrExt, OptionStringExt};
pub use overlay::Overlay;
pub use query_param_state::QueryParamState;
pub use query_suffix::filter_query_suffix;
pub use redirect::redirect_success;
pub use request_flags::{overlay_active, success_alert_requested};
pub use sha256_hex::sha256_hex;
pub use slugify_teletex::slugify_teletex;
pub use storage_url::StorageScheme;
#[cfg(not(feature = "database"))]
pub use storage_url::database_disabled_error;
