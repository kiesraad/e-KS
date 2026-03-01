//! Political group setup and maintenance flows.
mod extractors;
mod forms;
mod pages;
mod steps;
mod structs;

pub use forms::PoliticalGroupForm;
pub use pages::router;
pub use steps::PoliticalGroupSteps;
pub use structs::PoliticalGroup;
