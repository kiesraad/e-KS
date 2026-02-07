//! Shared constants used across the app.

/// Default date format
pub const DEFAULT_DATE_FORMAT: &str = "%d-%m-%Y";

/// Default datetime format
pub const DEFAULT_DATE_TIME_FORMAT: &str = "%d-%m-%Y %H:%M";

/// Default stream ID for the event store
pub const DEFAULT_STREAM_ID: uuid::Uuid = uuid::Uuid::from_u128(0);
