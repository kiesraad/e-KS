//! Person management and related routes.
mod actions;
mod extractors;
mod forms;
mod pages;
mod paths;

pub use forms::{AddressForm, PersonalDataFieldsForm, PersonalDataForm, RepresentativeForm};
pub use pages::router;
pub use paths::UpdatePersonPath;
// Only the guard test in this section reads this; see `view::context`.
#[cfg(test)]
pub use paths::PersonsPath;
