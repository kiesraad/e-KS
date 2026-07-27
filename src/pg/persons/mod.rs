//! Person management and related routes.
mod actions;
mod extractors;
mod forms;
mod pages;
mod paths;

pub use crate::structs::persons::{
    Person, PersonId, PersonPagination, PersonSort, PersonalData, Representative,
};
pub use forms::{AddressForm, PersonalDataFieldsForm, PersonalDataForm, RepresentativeForm};
pub use pages::router;
pub use paths::{PersonsPath, UpdatePersonPath};
