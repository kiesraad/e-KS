//! Loads runtime configuration from environment variables for AppState.
//! Used by AppState::new to construct service URLs and storage settings.

use std::env;

use crate::AppError;

const DEFAULT_STORAGE_URL: &str = "postgres://eks@localhost/eks";
const DEFAULT_TYPST_URL: &str = "http://localhost:8080";

/// Runtime configuration loaded from environment variables.
#[derive(Debug, Clone, Copy)]
pub struct Config {
    pub storage_url: &'static str,
    pub typst_url: &'static str,
}

/// Helper function to get environment variable or return an error
pub fn get_env(name: &'static str, _dev_default: &'static str) -> Result<String, AppError> {
    match env::var(name) {
        Ok(value) => Ok(value),
        #[cfg(feature = "dev-features")]
        Err(_) => Ok(_dev_default.to_string()),
        #[cfg(not(feature = "dev-features"))]
        Err(_) => Err(AppError::MissingEnvVar(name)),
    }
}

impl Config {
    pub fn from_env_with<F>(get: F) -> Result<Self, AppError>
    where
        F: Fn(&'static str, &'static str) -> Result<String, AppError>,
    {
        Ok(Self {
            storage_url: Box::leak(get("STORAGE_URL", DEFAULT_STORAGE_URL)?.into_boxed_str()),
            typst_url: Box::leak(get("TYPST_URL", DEFAULT_TYPST_URL)?.into_boxed_str()),
        })
    }

    pub fn from_env_with_typst_url(typst_url: Option<String>) -> Result<Self, AppError> {
        let storage_url = get_env("STORAGE_URL", DEFAULT_STORAGE_URL)?;
        let typst_url = match typst_url {
            Some(value) => value,
            None => get_env("TYPST_URL", DEFAULT_TYPST_URL)?,
        };

        Ok(Self {
            storage_url: Box::leak(storage_url.into_boxed_str()),
            typst_url: Box::leak(typst_url.into_boxed_str()),
        })
    }

    #[cfg(test)]
    pub fn new_test() -> Self {
        Self {
            storage_url: "memory://",
            typst_url: DEFAULT_TYPST_URL,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn loads_storage_url_from_provider() {
        let config = Config::from_env_with(|key, _default| match key {
            "STORAGE_URL" => Ok("postgres://example".to_string()),
            "TYPST_URL" => Ok("http://127.0.0.1:8080".to_string()),
            _ => Err(AppError::MissingEnvVar(key)),
        })
        .unwrap();

        assert_eq!(config.storage_url, "postgres://example");
    }

    #[test]
    fn returns_error_when_env_missing() {
        let key: &'static str = "STORAGE_URL";

        let err =
            Config::from_env_with(|_, _default| Err(AppError::MissingEnvVar(key))).unwrap_err();
        match err {
            AppError::MissingEnvVar(var) => assert_eq!(var, key),
            _ => panic!("unexpected error: {err:?}"),
        }
    }
}
