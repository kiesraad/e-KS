//! CSB monitoring: an operational overview listing each persisted
//! political-group stream with a few at-a-glance figures and, under the
//! `database` feature, its local cache position.
mod extractors;
mod pages;

pub use pages::{CsbMonitoringOverviewPath, router};
