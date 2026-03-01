//! Person management and related routes.
mod extractors;
mod forms;
mod pages;
mod structs;

pub use crate::QueryParamState;
pub use forms::{AddressForm, PersonForm, RepresentativeForm};
pub use pages::router;
pub use structs::{Person, PersonId, PersonPagination, PersonSort, Representative};
