mod configs;
mod districts;
mod macros;
mod public_session;
mod regions;
mod types;

pub use configs::ElectionConfig;
pub use districts::ElectoralDistrict;
pub use public_session::PublicSession;
pub use regions::{Province, WaterCouncil};
pub use types::ElectionType;

use macros::{define_districts, define_elections};
