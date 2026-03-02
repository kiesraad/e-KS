//! Tracing subscriber initialization for application logging.
//! Called during startup to set log filters and formatting.

use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

pub fn init() {
    let filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| format!("{}=debug,tower_http=info", env!("CARGO_CRATE_NAME")).into());

    println!("Logging filter: {filter}");

    tracing_subscriber::registry()
        .with(filter)
        .with(tracing_subscriber::fmt::layer())
        .init();
}
