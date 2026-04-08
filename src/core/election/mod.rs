mod config_macro;
mod configs;
mod districts;
mod regions;
mod types;

pub use configs::ElectionConfig;
pub use districts::ElectoralDistrict;
pub use regions::{Province, WaterCouncil};
pub use types::ElectionType;

use config_macro::define_elections;
