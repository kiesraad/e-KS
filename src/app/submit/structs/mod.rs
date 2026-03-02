use chrono::{Datelike, Timelike, Utc};
use serde::Serialize;

pub mod h1;

#[derive(Debug, Serialize)]
struct TypstDatetime {
    year: i32,
    month: u32,
    day: u32,
    hour: u32,
    minute: u32,
    second: u32,
}

impl TypstDatetime {
    fn now() -> Self {
        let now = Utc::now();
        Self {
            year: now.year(),
            month: now.month(),
            day: now.day(),
            hour: now.hour(),
            minute: now.minute(),
            second: now.second(),
        }
    }
}

#[derive(Debug, Serialize)]
struct TypstDate {
    year: i32,
    month: u32,
    day: u32,
}

impl From<crate::common::Date> for TypstDate {
    fn from(date: crate::common::Date) -> Self {
        Self {
            year: date.year(),
            month: date.month(),
            day: date.day(),
        }
    }
}
