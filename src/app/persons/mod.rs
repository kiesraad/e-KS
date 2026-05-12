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
pub use pages::{UpdatePersonPath, router};
pub use structs::{
    CANDIDATE_WARN_AGE, Person, PersonId, PersonPagination, PersonSort, PersonalData,
    Representative,
};
