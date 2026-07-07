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

#[cfg(test)]
pub use structs::EmptyAddressProblems;
pub use structs::{
    Address, BSN_NONE_CONFIRMATION, Bsn, BsnOrNoneConfirmed, COUNTRY_CODES, CountryCode,
    DateOfBirth, DisplayName, DutchAddress, FirstName, FormAction, FullName, Gender, HasSeverity,
    HouseNumber, HouseNumberAddition, InfoProblems, Initials, InternationalAddress,
    InternationalPostalCode, LastName, LastNamePrefix, LegalName, Locality, PlaceOfResidence,
    PostalCode, PotentialProblems, PreviousElectionResults, Problematic, Problems, Severity,
    StateOrProvince, StreetName, UtcDateTime, WithProblems,
};

pub use pages::{
    HideDownloadWarningPath, IndexPath, LoginStartPath, SelectElectionPath, SwitchElectionPath,
    SwitchLanguagePath, auth_failure_response, not_found, public_router, router,
    session_only_router, wellknown_router,
};
