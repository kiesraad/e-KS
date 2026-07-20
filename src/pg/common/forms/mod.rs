mod address;
mod name;
mod select_election;
mod switch_election;

pub use address::{DutchAddressForm, InternationalAddressForm};
pub use name::{FullNameForm, MinimalNameForm};
pub use select_election::SelectElectionForm;
pub use switch_election::SwitchElectionForm;
