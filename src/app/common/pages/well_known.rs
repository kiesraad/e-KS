use axum::response::IntoResponse;
use axum_extra::routing::TypedPath;
use chrono::{DateTime, NaiveDate, NaiveDateTime, NaiveTime, SecondsFormat, Utc};
use serde::Deserialize;

use crate::AppError;

#[derive(TypedPath, Deserialize)]
#[typed_path("/.well-known/{file_name}")]
pub(super) struct WellKnownEntry {
    file_name: String,
}

pub(super) async fn index(WellKnownEntry { file_name }: WellKnownEntry) -> impl IntoResponse {
    match file_name.as_str() {
        "security.txt" => Ok(security_text()),
        _ => Err(AppError::GenericNotFound),
    }
}

const SECURITY_EXPIRATION: DateTime<Utc> = DateTime::from_naive_utc_and_offset(
    NaiveDateTime::new(
        NaiveDate::from_ymd_opt(2026, 7, 25).unwrap(),
        NaiveTime::MIN,
    ),
    Utc,
);

fn security_text() -> String {
    let date_str = SECURITY_EXPIRATION.to_rfc3339_opts(SecondsFormat::Secs, true);

    format!(
        "Contact: mailto:security@kiesraad.nl
Expires: {date_str}
Preferred-Languages: en, nl
Canonical: https://kandidaatstellen.kiesraad.nl/.well-known/security.txt
Policy: https://code.overheid.nl/Kiesraad/e-KS/src/branch/main/SECURITY.md
Hiring: https://www.werkenvoornederland.nl
CSAF: https://advisories.ncsc.nl/.well-known/csaf/provider-metadata.json
"
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::test_utils::response_body_string;
    use chrono::{Months, Utc};

    #[tokio::test]
    async fn security_renders_text() {
        let body = index(WellKnownEntry {
            file_name: "security.txt".to_string(),
        })
        .await
        .into_response();

        let body = response_body_string(body).await;
        assert_eq!(body, security_text());
    }

    #[test]
    fn security_is_not_stale() {
        // This tests that EXPIRATION is in the future, and,
        // per RFC9116, is not more than one year in the future
        assert!(SECURITY_EXPIRATION > Utc::now());
        assert!(SECURITY_EXPIRATION < Utc::now() + Months::new(12));

        // Nice idea from Michiel: make the unit test deliberately flaky
        // if the expiration date is soon to expire. This will definitely
        // show up in CI, but can easily be squelched by re-trying so it won't
        // block whatever else needs attention.
        if SECURITY_EXPIRATION < Utc::now() + Months::new(3) {
            use rand::RngExt;
            if rand::rng().random_bool(1.0 / 3.0) {
                panic!("SECURITY_EXPIRATION will expire within three months, please update it");
            }
        }
    }
}
