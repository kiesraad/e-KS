//! Shared domain primitives and common pages.
//!
//! Scope: reusable value types (names, addresses, dates), shared extractors,
//! and common pages such as index and language switching.
mod extractors;
mod pages;
mod structs;

pub use structs::{
    Bsn, COUNTRY_CODES, CountryCode, Date, DisplayName, DutchAddress, DutchAddressForm, FirstName,
    FormAction, FullName, FullNameForm, Gender, HouseNumber, HouseNumberAddition, Initials,
    LastName, LastNamePrefix, LegalName, Locality, PlaceOfResidence, PostalCode, StreetName,
    UtcDateTime,
};

pub use pages::{IndexPath, SwitchLanguagePath, not_found, router};
