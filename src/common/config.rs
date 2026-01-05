//! Loads runtime configuration from environment variables for AppState.
//! Used by AppState::new to construct service URLs, database settings, etc.

use std::env;

use crate::{AppError, ElectionConfig, ElectoralDistrict};

#[derive(Debug)]
pub struct Config {
    pub database_url: String,
    pub election: ElectionConfig,
}

/// Helper function to get environment variable or return an error
fn get_env(name: &'static str, _dev_default: &'static str) -> Result<String, AppError> {
    match env::var(name) {
        Ok(value) => Ok(value),
        #[cfg(feature = "dev-features")]
        Err(_) => Ok(_dev_default.to_string()),
        #[cfg(not(feature = "dev-features"))]
        Err(_) => Err(AppError::MissingEnvVar(name)),
    }
}

impl Config {
    pub fn from_env() -> Result<Self, AppError> {
        Self::from_env_with(get_env)
    }

    pub fn from_env_with<F>(get: F) -> Result<Self, AppError>
    where
        F: Fn(&'static str, &'static str) -> Result<String, AppError>,
    {
        Ok(Self {
            database_url: get("DATABASE_URL", "postgres://eks@localhost/eks")?,
            election: ElectionConfig::EK2027,
        })
    }

    pub fn get_districts(&self) -> &'static [ElectoralDistrict] {
        self.election.electoral_districts()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn loads_database_url_from_provider() {
        let config = Config::from_env_with(|key, _default| match key {
            "DATABASE_URL" => Ok("postgres://example".to_string()),
            _ => Err(AppError::MissingEnvVar(key)),
        })
        .unwrap();

        assert_eq!(config.database_url, "postgres://example");
    }

    #[test]
    fn returns_error_when_env_missing() {
        let key: &'static str = "DATABASE_URL";

        let err =
            Config::from_env_with(|_, _default| Err(AppError::MissingEnvVar(key))).unwrap_err();
        match err {
            AppError::MissingEnvVar(var) => assert_eq!(var, key),
            _ => panic!("unexpected error: {err:?}"),
        }
    }
}
