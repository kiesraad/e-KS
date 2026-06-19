//! Shared domain primitives and common pages.
//!
//! Scope: reusable value types (names, addresses, dates), shared extractors,
//! and common pages such as index and language switching.
mod extractors;
mod forms;
mod pages;
mod structs;

pub use forms::{
    DutchAddressForm, FullNameForm, InternationalAddressForm, MinimalNameForm, SelectElectionForm,
    SwitchElectionForm,
};
pub use structs::{
    Address, BSN_NONE_CONFIRMATION, Bsn, BsnOrNoneConfirmed, COUNTRY_CODES, CountryCode,
    DateOfBirth, DisplayName, DutchAddress, EmptyAddressProblems, FirstName, FormAction, FullName,
    Gender, HasSeverity, HouseNumber, HouseNumberAddition, InfoProblems, Initials,
    InternationalAddress, InternationalPostalCode, LastName, LastNamePrefix, LegalName, Locality,
    PlaceOfResidence, PostalCode, PotentialProblems, PreviousElectionResults, Problematic,
    Problems, Severity, StateOrProvince, StreetName, UtcDateTime, WithProblems,
};

pub use pages::{
    IndexPath, SelectElectionPath, SwitchElectionPath, SwitchLanguagePath, not_found, router,
    session_only_router, wellknown_router,
};
