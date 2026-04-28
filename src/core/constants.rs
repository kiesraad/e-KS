//! Shared constants used across the app.

use chrono_tz::{Europe, Tz};

/// Default date format
pub const DEFAULT_DATE_FORMAT: &str = "%d-%m-%Y";

/// Default datetime format
pub const DEFAULT_DATE_TIME_FORMAT: &str = "%d-%m-%Y %H:%M";

pub const DEFAULT_TIMEZONE: &Tz = &Europe::Amsterdam;
