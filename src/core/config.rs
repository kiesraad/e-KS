//! Loads runtime configuration from environment variables for AppState.
//! Used by AppState::new to construct service URLs and storage settings.

use std::env;

use secrecy::SecretString;

use crate::AppError;

#[cfg(feature = "dev-features")]
mod dev_defaults {
    use super::*;

    #[cfg(feature = "database")]
    pub(super) const STORAGE_URL: &str = "postgres://eks@localhost/eks";

    #[cfg(not(feature = "database"))]
    pub(super) const STORAGE_URL: &str = "memory://ephemeral";

    pub(super) const TYPST_URL: &str = "http://localhost:8080";

    pub(super) const BAG_SERVICE_URL: &str = "http://localhost:8090";

    pub(super) const ID_DERIVATION_KEY: &str = "eks-dev-id-derivation-key-not-for-production";

    pub(super) const DEFAULT_ENCRYPTION_DERIVATION_KEY: &str =
        "eks-dev-encryption-derivation-key-not-for-production";

    pub(super) fn lookup(name: &'static str) -> Result<String, env::VarError> {
        std::collections::HashMap::from([
            ("STORAGE_URL", STORAGE_URL),
            ("TYPST_URL", TYPST_URL),
            ("BAG_SERVICE_URL", BAG_SERVICE_URL),
            ("ID_DERIVATION_KEY", ID_DERIVATION_KEY),
            (
                "ENCRYPTION_DERIVATION_KEY",
                DEFAULT_ENCRYPTION_DERIVATION_KEY,
            ),
        ])
        .get(name)
        .map(|value| (*value).to_string())
        .ok_or(env::VarError::NotPresent)
    }
}

/// Runtime configuration loaded from environment variables.
#[derive(Debug, Clone)]
pub struct Config {
    pub storage_url: String,
    pub typst_url: String,
    pub id_derivation_key: SecretString,
    pub encryption_derivation_key: SecretString,
}

/// Helper function to get environment variable or return an error
pub fn get_env(name: &'static str) -> Result<String, AppError> {
    get_env_with(name, &mut |key| env::var(key))
}

fn get_env_with<F>(name: &'static str, lookup: &mut F) -> Result<String, AppError>
where
    F: FnMut(&'static str) -> Result<String, env::VarError>,
{
    match lookup(name) {
        Ok(value) => Ok(value),
        #[cfg(feature = "dev-features")]
        Err(_) => Ok(dev_defaults::lookup(name).map_err(|_| AppError::MissingEnvVar(name))?),
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
        let storage_url = get_env_with("STORAGE_URL", &mut lookup)?;
        let typst_url = match typst_url {
            Some(value) => value,
            None => get_env_with("TYPST_URL", &mut lookup)?,
        };
        let id_derivation_key = get_env_with("ID_DERIVATION_KEY", &mut lookup)?;

        let encryption_derivation_key = get_env_with("ENCRYPTION_DERIVATION_KEY", &mut lookup)?;

        Ok(Self {
            storage_url,
            typst_url,
            id_derivation_key: SecretString::from(id_derivation_key),
            encryption_derivation_key: SecretString::from(encryption_derivation_key),
        })
    }

    #[cfg(test)]
    pub fn new_test() -> Self {
        Self {
            storage_url: "memory://".to_string(),
            typst_url: "http://localhost:8080".to_string(),
            id_derivation_key: SecretString::from("test-secret-123"),
            encryption_derivation_key: SecretString::from("test-encryption-secret-123"),
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

        let value = get_env_with("TEST_CONFIG_ENV", &mut lookup).expect("env value");

        assert_eq!(value, "present");
    }

    #[test]
    fn from_env_uses_env_values() {
        let map = HashMap::from([
            ("STORAGE_URL", "memory://test"),
            ("TYPST_URL", "http://typst.test"),
            ("ID_DERIVATION_KEY", "test-secret-123"),
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
            ("ID_DERIVATION_KEY", "test-secret-123"),
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

        let value = get_env_with("ID_DERIVATION_KEY", &mut lookup).expect("dev default");

        assert_eq!(value, dev_defaults::ID_DERIVATION_KEY);
    }

    #[cfg(not(feature = "dev-features"))]
    #[test]
    fn get_env_errors_when_missing_without_dev_features() {
        let map = HashMap::new();
        let mut lookup = lookup_from(&map);

        let err = get_env_with("TEST_CONFIG_MISSING", &mut lookup).expect_err("missing env");

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

        assert_eq!(config.storage_url, dev_defaults::STORAGE_URL);
        assert_eq!(config.typst_url, dev_defaults::TYPST_URL);
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
