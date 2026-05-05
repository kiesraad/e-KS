//! Tracing subscriber initialization for application logging.
//! Called during startup to set log filters and formatting.
//!
//! Format is selected at runtime via `LOG_FORMAT`:
//! - `LOG_FORMAT=json` emits one JSON object per event on stdout (intended for
//!   ingestion by a Loki agent).
//! - Anything else (or unset) keeps the human-readable pretty formatter.

use tracing_subscriber::{EnvFilter, layer::SubscriberExt, util::SubscriberInitExt};

pub fn init() {
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| {
        format!(
            "{}=info,tower_http=info,auth_service=info",
            env!("CARGO_CRATE_NAME")
        )
        .into()
    });

    let json = matches!(
        std::env::var("LOG_FORMAT").as_deref(),
        Ok("json") | Ok("JSON")
    );

    println!(
        "Logging filter: {filter} (format: {})",
        if json { "json" } else { "pretty" }
    );

    let registry = tracing_subscriber::registry().with(filter);

    if json {
        registry
            .with(
                tracing_subscriber::fmt::layer()
                    .json()
                    .flatten_event(true)
                    .with_current_span(true)
                    .with_span_list(false),
            )
            .init();
    } else {
        registry.with(tracing_subscriber::fmt::layer()).init();
    }
}
