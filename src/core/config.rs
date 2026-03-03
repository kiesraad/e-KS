//! Loads runtime configuration from environment variables for AppState.
//! Used by AppState::new to construct service URLs and storage settings.

use std::env;

use crate::AppError;

#[cfg(feature = "database")]
const DEFAULT_STORAGE_URL: &str = "postgres://eks@localhost/eks";

#[cfg(not(feature = "database"))]
const DEFAULT_STORAGE_URL: &str = "memory://ephemeral";

const DEFAULT_TYPST_URL: &str = "http://localhost:8080";

/// Runtime configuration loaded from environment variables.
#[derive(Debug, Clone)]
pub struct Config {
    pub storage_url: String,
    pub typst_url: String,
}

/// Helper function to get environment variable or return an error
pub fn get_env(name: &'static str, _dev_default: &'static str) -> Result<String, AppError> {
    get_env_with(name, _dev_default, &mut |key| env::var(key))
}

fn get_env_with<F>(
    name: &'static str,
    _dev_default: &'static str,
    lookup: &mut F,
) -> Result<String, AppError>
where
    F: FnMut(&'static str) -> Result<String, env::VarError>,
{
    match lookup(name) {
        Ok(value) => Ok(value),
        #[cfg(feature = "dev-features")]
        Err(_) => Ok(_dev_default.to_string()),
        #[cfg(not(feature = "dev-features"))]
        Err(_) => Err(AppError::MissingEnvVar(name)),
    }
}

impl Config {
    pub fn from_env(typst_url: Option<String>) -> Result<Self, AppError> {
        Self::from_env_with(typst_url, env::var)
    }

    fn from_env_with<F>(typst_url: Option<String>, mut lookup: F) -> Result<Self, AppError>
    where
        F: FnMut(&'static str) -> Result<String, env::VarError>,
    {
        let storage_url = get_env_with("STORAGE_URL", DEFAULT_STORAGE_URL, &mut lookup)?;
        let typst_url = match typst_url {
            Some(value) => value,
            None => get_env_with("TYPST_URL", DEFAULT_TYPST_URL, &mut lookup)?,
        };

        Ok(Self {
            storage_url,
            typst_url,
        })
    }

    #[cfg(test)]
    pub fn new_test() -> Self {
        Self {
            storage_url: "memory://".to_string(),
            typst_url: DEFAULT_TYPST_URL.to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn lookup_from(
        map: &HashMap<&'static str, &'static str>,
    ) -> impl FnMut(&'static str) -> Result<String, env::VarError> {
        move |key| {
            map.get(key)
                .map(|value| (*value).to_string())
                .ok_or(env::VarError::NotPresent)
        }
    }

    #[test]
    fn get_env_returns_value_when_set() {
        let map = HashMap::from([("TEST_CONFIG_ENV", "present")]);
        let mut lookup = lookup_from(&map);

        let value = get_env_with("TEST_CONFIG_ENV", "fallback", &mut lookup).expect("env value");

        assert_eq!(value, "present");
    }

    #[test]
    fn from_env_uses_env_values() {
        let map = HashMap::from([
            ("STORAGE_URL", "memory://test"),
            ("TYPST_URL", "http://typst.test"),
        ]);
        let lookup = lookup_from(&map);

        let config = Config::from_env_with(None, lookup).expect("config");

        assert_eq!(config.storage_url, "memory://test");
        assert_eq!(config.typst_url, "http://typst.test");
    }

    #[test]
    fn from_env_prefers_override() {
        let map = HashMap::from([
            ("STORAGE_URL", "memory://override"),
            ("TYPST_URL", "http://typst.env"),
        ]);
        let lookup = lookup_from(&map);

        let config = Config::from_env_with(Some("http://typst.override".to_string()), lookup)
            .expect("config");

        assert_eq!(config.storage_url, "memory://override");
        assert_eq!(config.typst_url, "http://typst.override");
    }

    #[cfg(feature = "dev-features")]
    #[test]
    fn get_env_returns_default_when_missing_in_dev_features() {
        let map = HashMap::new();
        let mut lookup = lookup_from(&map);

        let value =
            get_env_with("TEST_CONFIG_MISSING", "default", &mut lookup).expect("dev default");

        assert_eq!(value, "default");
    }

    #[cfg(not(feature = "dev-features"))]
    #[test]
    fn get_env_errors_when_missing_without_dev_features() {
        let map = HashMap::new();
        let mut lookup = lookup_from(&map);

        let err =
            get_env_with("TEST_CONFIG_MISSING", "default", &mut lookup).expect_err("missing env");

        assert_eq!(
            err.to_string(),
            AppError::MissingEnvVar("TEST_CONFIG_MISSING").to_string()
        );
    }

    #[cfg(feature = "dev-features")]
    #[test]
    fn from_env_uses_defaults_in_dev_features() {
        let map = HashMap::new();
        let lookup = lookup_from(&map);

        let config = Config::from_env_with(None, lookup).expect("dev defaults");

        assert_eq!(config.storage_url, DEFAULT_STORAGE_URL);
        assert_eq!(config.typst_url, DEFAULT_TYPST_URL);
    }

    #[cfg(not(feature = "dev-features"))]
    #[test]
    fn from_env_errors_when_storage_missing_without_dev_features() {
        let map = HashMap::new();
        let lookup = lookup_from(&map);

        let err = Config::from_env_with(None, lookup).expect_err("missing storage");

        assert_eq!(
            err.to_string(),
            AppError::MissingEnvVar("STORAGE_URL").to_string()
        );
    }
}
