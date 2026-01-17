mod address_form;
mod gender;
mod person;
mod person_form;
mod person_sort;

pub use address_form::AddressForm;
pub use gender::Gender;
pub use person::{Person, PersonId};
pub use person_form::PersonForm;
pub use person_sort::PersonSort;

pub trait PersonIdPath {
    fn person_id(&self) -> PersonId;
}
