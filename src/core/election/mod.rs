mod configs;
mod districts;
mod macros;
mod regions;
mod types;

pub use configs::ElectionConfig;
pub use districts::ElectoralDistrict;
pub use regions::{Province, WaterCouncil};
pub use types::ElectionType;

use macros::{define_districts, define_elections};
