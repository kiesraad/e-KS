//! Person management and related routes.
mod extractors;
mod forms;
mod pages;

pub use crate::structs::persons::{
    Person, PersonId, PersonPagination, PersonSort, PersonalData, Representative,
};
pub use forms::{AddressForm, PersonalDataFieldsForm, PersonalDataForm, RepresentativeForm};
pub use pages::{PersonsPath, UpdatePersonPath, router};
