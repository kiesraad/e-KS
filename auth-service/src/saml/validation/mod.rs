//! SAML receive-side validators.
//!
//! Implements the DV processing rules from eID §7.6/§7.7 split across:
//! - `artifact_response`: SOAP-wrapped ArtifactResponse validation (§7.6.1).
//! - `response`: inner Response validation (§7.6.2).
//! - `assertion`: Assertion validation and claim extraction (§7.6.3).
//! - `logout_response`: LogoutResponse structural validation (§7.7.2).
//! - `helpers`: the shared `helpers::Validator` context and checks used by
//!   more than one of the above.
//!
//! The SOAP envelope is unwrapped by
//! [`bindings::soap::unwrap_soap`](crate::bindings::soap::unwrap_soap), and the
//! Level-of-Assurance domain type lives in [`saml::loa`](crate::saml::loa).

mod artifact_response;
mod assertion;
mod helpers;
mod logout_response;
mod response;

pub use artifact_response::{ValidateArtifactResponseOpts, validate_artifact_response_at};
pub use assertion::{Claims, ValidateAssertionOpts, validate_assertion_at};
pub use logout_response::{LogoutResponseFields, validate_logout_response};
pub use response::{ValidateResponseOpts, validate_response_at};
