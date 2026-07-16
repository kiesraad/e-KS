//! Shared constants used across the app.

use chrono_tz::{Europe, Tz};

/// Default date format
pub const DEFAULT_DATE_FORMAT: &str = "%d-%m-%Y";

/// Default time format
pub const DEFAULT_TIME_FORMAT: &str = "%H:%M";

/// Default datetime format
pub const DEFAULT_DATE_TIME_FORMAT: &str = "%d-%m-%Y %H:%M";

/// Default datetime format with seconds
pub const DATE_TIME_SECONDS_FORMAT: &str = "%d-%m-%Y %H:%M:%S";

pub const DEFAULT_TIMEZONE: &Tz = &Europe::Amsterdam;

pub const MAX_CANDIDATES: usize = 80;
