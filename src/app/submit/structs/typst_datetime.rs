use chrono::{Datelike, Timelike, Utc};
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct TypstDatetime {
    pub year: i32,
    pub month: u32,
    pub day: u32,
    pub hour: u32,
    pub minute: u32,
    pub second: u32,
}

impl TypstDatetime {
    pub fn now() -> Self {
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
pub struct TypstDate {
    pub year: i32,
    pub month: u32,
    pub day: u32,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn date_from_common_date_copies_components() {
        let input: crate::common::Date = "15-03-2001".parse().expect("date");
        let date = TypstDate::from(input);

        assert_eq!(date.year, 2001);
        assert_eq!(date.month, 3);
        assert_eq!(date.day, 15);
    }
}
