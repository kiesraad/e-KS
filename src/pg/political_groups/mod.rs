//! Political group setup and maintenance flows.
mod extractors;
mod forms;
mod pages;
mod steps;

pub use crate::structs::political_groups::PoliticalGroup;
pub use forms::PoliticalGroupForm;
pub use pages::{PoliticalGroupUpdatePath, router};
pub use steps::PoliticalGroupSteps;
