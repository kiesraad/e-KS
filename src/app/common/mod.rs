//! Shared domain primitives and common pages.
//!
//! Scope: reusable value types (names, addresses, dates), shared extractors,
//! and common pages such as index and language switching.
mod extractors;
mod forms;
mod pages;
mod structs;

pub use forms::{DutchAddressForm, FullNameForm};
pub use structs::{
    BSN_NONE_CONFIRMATION, Bsn, BsnOrNoneConfirmed, COUNTRY_CODES, CountryCode, Date, DisplayName,
    DutchAddress, FirstName, FormAction, FullName, Gender, HouseNumber, HouseNumberAddition,
    Initials, LastName, LastNamePrefix, LegalName, Locality, PlaceOfResidence, PostalCode,
    StreetName, UtcDateTime,
};

pub use pages::{IndexPath, SwitchLanguagePath, not_found, router};
