use std::str::FromStr;

use crate::{form::ValidationError, transparent_string};

pub const RVIG_COUNTRY_CODES_URL: &str = "https://publicaties.rvig.nl/media/13286/download";

transparent_string! {
    pub struct CountryCode(String);
}

impl FromStr for CountryCode {
    type Err = ValidationError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let trimmed_value = value.trim().to_uppercase();

        if !super::COUNTRY_CODES.contains(&trimmed_value.as_str()) {
            return Err(ValidationError::InvalidValue);
        }

        Ok(CountryCode(trimmed_value))
    }
}

impl CountryCode {
    pub fn is_nl(&self) -> bool {
        self.0 == "NL"
    }
}

#[cfg(test)]
mod tests {
    use reqwest::{StatusCode, header};

    use super::*;

    #[tokio::test]
    async fn rvig_country_code_url_not_dead() -> Result<(), reqwest::Error> {
        let response = reqwest::Client::new()
            .get(RVIG_COUNTRY_CODES_URL)
            .send()
            .await?;

        assert_eq!(response.status(), StatusCode::OK);

        let content_type = response
            .headers()
            .get(header::CONTENT_TYPE)
            .expect("no content type header")
            .to_str()
            .ok()
            .unwrap();
        assert_eq!(content_type, "application/pdf");

        Ok(())
    }
}
