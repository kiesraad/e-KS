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

static LOCALE_COOKIE_NAME: &str = "LANGUAGE";
