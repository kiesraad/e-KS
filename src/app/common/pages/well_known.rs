use axum_extra::routing::TypedPath;
use chrono::{DateTime, NaiveDate, NaiveDateTime, NaiveTime, SecondsFormat, Utc};
use serde::Deserialize;

#[derive(TypedPath, Deserialize)]
#[typed_path("/security.txt")]
pub struct SecurityTxt {}

const SECURITY_EXPIRATION: DateTime<Utc> = DateTime::from_naive_utc_and_offset(
    NaiveDateTime::new(NaiveDate::from_ymd_opt(2027, 2, 1).unwrap(), NaiveTime::MIN),
    Utc,
);

/// Emit a security.txt file, for background information see <https://github.com/securitytxt/security-txt>
pub(super) async fn security_txt(_: SecurityTxt) -> String {
    let date_str = SECURITY_EXPIRATION.to_rfc3339_opts(SecondsFormat::Secs, true);

    format!(
        "Expires: {date_str}
Canonical: https://kandidaatstellen.kiesraad.nl/.well-known/security.txt

Policy: https://code.overheid.nl/Kiesraad/e-KS/src/branch/main/SECURITY.md

Contact: mailto:security@kiesraad.nl
Preferred-Languages: en, nl

Hiring: https://www.werkenvoornederland.nl
"
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::test_utils::response_body_string;
    use axum::response::IntoResponse;
    use chrono::{Months, Utc};

    #[tokio::test]
    async fn security_renders_text() {
        // This text is almost useless, except that it forces (at compile time, most likely)
        // that security.txt is a text/plain.
        let text = security_txt(SecurityTxt {}).await;
        let body = response_body_string(text.clone().into_response()).await;
        assert_eq!(body, text);
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
