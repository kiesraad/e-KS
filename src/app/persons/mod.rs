//! Person management and related routes.
mod extractors;
mod forms;
mod pages;
mod structs;

pub use crate::QueryParamState;
pub use forms::{
    AddressForm, PersonalDataFieldsForm, PersonalDataForm, RepresentativeFieldsForm,
    RepresentativeForm,
};
pub use pages::{PersonsPath, UpdatePersonPath, router};
pub use structs::{Person, PersonId, PersonPagination, PersonSort, PersonalData, Representative};
