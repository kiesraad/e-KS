//! Shared domain primitives and common pages.
//!
//! Scope: re-exports of the shared value types (`src/structs/common`), shared
//! extractors, and common pages such as index and language switching.
mod extractors;
mod forms;
mod pages;
mod paths;

pub use forms::{
    DutchAddressForm, FullNameForm, InternationalAddressForm, MinimalNameForm, SelectElectionForm,
    SwitchElectionForm,
};

#[cfg(test)]
pub use crate::structs::common::EmptyAddressProblems;
pub use crate::structs::common::{
    Address, BSN_NONE_CONFIRMATION, Bsn, BsnOrNoneConfirmed, COUNTRY_CODES, CountryCode,
    DateOfBirth, DisplayName, DutchAddress, FirstName, FormAction, FullName, Gender, HasSeverity,
    HouseNumber, HouseNumberAddition, InfoProblems, Initials, InternationalAddress,
    InternationalPostalCode, LastName, LastNamePrefix, LegalName, Locality, PlaceOfResidence,
    PostalCode, PotentialProblems, PreviousElectionResults, Problematic, Problems,
    RVIG_COUNTRY_CODES_URL, Severity, StateOrProvince, StreetName, UtcDateTime, WithProblems,
};

pub use pages::{not_found, public_router, router, session_only_router, wellknown_router};
pub use paths::{
    HideDownloadWarningPath, IndexPath, LoginStartPath, LogoutPath, SelectElectionPath,
    SwitchElectionPath, SwitchLanguagePath,
};
