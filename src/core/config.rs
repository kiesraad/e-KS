//! Loads runtime configuration from environment variables for AppState.
//! Used by AppState::new to construct service URLs and storage settings.

use std::{env, path::PathBuf};

use secrecy::SecretString;

use crate::AppError;

#[cfg(feature = "dev-features")]
mod dev_defaults {
    #[cfg(feature = "database")]
    pub(super) const STORAGE_URL: &str = "postgres://eks@localhost/eks";

    #[cfg(not(feature = "database"))]
    pub(super) const STORAGE_URL: &str = "memory://ephemeral";

    #[cfg(not(feature = "embed-typst"))]
    pub(super) const TYPST_URL: &str = "http://localhost:8080";

    pub(super) const ID_DERIVATION_KEY: &str = "eks-dev-id-derivation-key-not-for-production";

    pub(super) const DEFAULT_ENCRYPTION_DERIVATION_KEY: &str =
        "eks-dev-encryption-derivation-key-not-for-production";

    pub(super) fn lookup(name: &'static str) -> Result<String, std::env::VarError> {
        std::collections::HashMap::from([
            ("STORAGE_URL", STORAGE_URL),
            #[cfg(not(feature = "embed-typst"))]
            ("TYPST_URL", TYPST_URL),
            ("ID_DERIVATION_KEY", ID_DERIVATION_KEY),
            (
                "ENCRYPTION_DERIVATION_KEY",
                DEFAULT_ENCRYPTION_DERIVATION_KEY,
            ),
        ])
        .get(name)
        .map(|value| (*value).to_string())
        .ok_or(std::env::VarError::NotPresent)
    }
}

/// TLS configuration for serving HTTPS via rustls.
#[derive(Debug, Clone)]
pub struct TlsConfig {
    pub cert_path: PathBuf,
    pub key_path: PathBuf,
}

/// Runtime configuration loaded from environment variables.
#[derive(Debug, Clone)]
pub struct Config {
    pub storage_url: SecretString,
    #[cfg(not(feature = "embed-typst"))]
    pub typst_url: String,
    pub id_derivation_key: SecretString,
    pub encryption_derivation_key: SecretString,
    pub tls: Option<TlsConfig>,
    /// Short identifier of the server this instance runs on (e.g. "S1"),
    /// rendered next to the version in the layout footer.
    pub server_name: Option<String>,
    /// When set, every request must carry an `x-eks-key` header whose value
    /// matches this secret. Intended for gating the app behind a known
    /// upstream (e.g. a load balancer that injects the header).
    pub eks_key: Option<SecretString>,
    /// When true, opts this instance out of the live auth-service (so
    /// `AuthServiceState::new_empty` is used instead of
    /// `AuthServiceState::new_from_env`, skipping the startup IdP-metadata
    /// fetch). Intended for environments that only ever use the `/dev/login`
    /// bypass and must boot without outbound connectivity, e.g. the Playwright
    /// container. Set via `DISABLE_AUTH_SERVICE` (`1`, `true`, or `yes`,
    /// case-insensitive); anything else leaves the auth-service enabled.
    pub disable_auth_service: bool,
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
    pub fn from_env() -> Result<Self, AppError> {
        Self::from_env_with(env::var)
    }

    fn from_env_with<F>(mut lookup: F) -> Result<Self, AppError>
    where
        F: FnMut(&'static str) -> Result<String, env::VarError>,
    {
        let storage_url = get_env_with("STORAGE_URL", &mut lookup)?;
        #[cfg(not(feature = "embed-typst"))]
        let typst_url = get_env_with("TYPST_URL", &mut lookup)?;
        let id_derivation_key = get_env_with("ID_DERIVATION_KEY", &mut lookup)?;

        let encryption_derivation_key = get_env_with("ENCRYPTION_DERIVATION_KEY", &mut lookup)?;

        let tls = match (lookup("TLS_CERT_PATH").ok(), lookup("TLS_KEY_PATH").ok()) {
            (Some(cert), Some(key)) => Some(TlsConfig {
                cert_path: PathBuf::from(cert),
                key_path: PathBuf::from(key),
            }),
            (None, None) => None,
            _ => {
                return Err(AppError::ConfigLoadError(
                    "TLS_CERT_PATH and TLS_KEY_PATH must both be set, or both unset".to_string(),
                ));
            }
        };

        let server_name = lookup("SERVER_NAME").ok().filter(|s| !s.is_empty());

        let eks_key = lookup("EKS_KEY")
            .ok()
            .filter(|s| !s.is_empty())
            .map(SecretString::from);

        let disable_auth_service = lookup("DISABLE_AUTH_SERVICE").is_ok_and(|value| {
            matches!(
                value.trim().to_ascii_lowercase().as_str(),
                "1" | "true" | "yes"
            )
        });

        Ok(Self {
            storage_url: SecretString::from(storage_url),
            #[cfg(not(feature = "embed-typst"))]
            typst_url,
            id_derivation_key: SecretString::from(id_derivation_key),
            encryption_derivation_key: SecretString::from(encryption_derivation_key),
            tls,
            server_name,
            eks_key,
            disable_auth_service,
        })
    }

    #[cfg(test)]
    pub fn new_test() -> Self {
        Self {
            storage_url: SecretString::from("memory://"),
            #[cfg(not(feature = "embed-typst"))]
            typst_url: "http://localhost:8080".to_string(),
            id_derivation_key: SecretString::from("test-secret-123"),
            encryption_derivation_key: SecretString::from("test-encryption-secret-123"),
            tls: None,
            server_name: None,
            eks_key: None,
            disable_auth_service: false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use secrecy::ExposeSecret;
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

    #[cfg(not(feature = "embed-typst"))]
    #[test]
    fn from_env_uses_env_values() {
        let map = HashMap::from([
            ("STORAGE_URL", "memory://test"),
            ("TYPST_URL", "http://typst.test"),
            ("ID_DERIVATION_KEY", "test-secret-123"),
        ]);
        let lookup = lookup_from(&map);

        let config = Config::from_env_with(lookup).expect("config");

        assert_eq!(config.storage_url.expose_secret(), "memory://test");
        assert_eq!(config.typst_url, "http://typst.test");
    }

    #[cfg(feature = "embed-typst")]
    #[test]
    fn from_env_uses_env_values_with_embed_typst() {
        let map = HashMap::from([
            ("STORAGE_URL", "memory://test"),
            ("ID_DERIVATION_KEY", "test-secret-123"),
        ]);
        let lookup = lookup_from(&map);

        let config = Config::from_env_with(lookup).expect("config");

        assert_eq!(config.storage_url.expose_secret(), "memory://test");
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

    #[cfg(all(feature = "dev-features", not(feature = "embed-typst")))]
    #[test]
    fn from_env_uses_defaults_in_dev_features() {
        let map = HashMap::new();
        let lookup = lookup_from(&map);

        let config = Config::from_env_with(lookup).expect("dev defaults");

        assert_eq!(
            config.storage_url.expose_secret(),
            dev_defaults::STORAGE_URL
        );
        assert_eq!(config.typst_url, dev_defaults::TYPST_URL);
    }

    #[cfg(all(feature = "dev-features", feature = "embed-typst"))]
    #[test]
    fn from_env_uses_defaults_in_dev_features() {
        let map = HashMap::new();
        let lookup = lookup_from(&map);

        let config = Config::from_env_with(lookup).expect("dev defaults");

        assert_eq!(
            config.storage_url.expose_secret(),
            dev_defaults::STORAGE_URL
        );
    }

    #[cfg(not(feature = "dev-features"))]
    #[test]
    fn from_env_errors_when_storage_missing_without_dev_features() {
        let map = HashMap::new();
        let lookup = lookup_from(&map);

        let err = Config::from_env_with(lookup).expect_err("missing storage");

        assert_eq!(
            err.to_string(),
            AppError::MissingEnvVar("STORAGE_URL").to_string()
        );
    }

    #[test]
    fn from_env_returns_no_tls_when_unset() {
        let map = HashMap::new();
        let lookup = lookup_from(&map);

        let config = Config::from_env_with(lookup).expect("config");

        assert!(config.tls.is_none());
    }

    #[test]
    fn from_env_returns_tls_when_both_set() {
        let map = HashMap::from([
            ("TLS_CERT_PATH", "/etc/tls/cert.pem"),
            ("TLS_KEY_PATH", "/etc/tls/key.pem"),
        ]);
        let lookup = lookup_from(&map);

        let config = Config::from_env_with(lookup).expect("config");
        let tls = config.tls.expect("tls present");

        assert_eq!(tls.cert_path, std::path::PathBuf::from("/etc/tls/cert.pem"));
        assert_eq!(tls.key_path, std::path::PathBuf::from("/etc/tls/key.pem"));
    }

    #[test]
    fn from_env_errors_when_only_tls_cert_set() {
        let map = HashMap::from([("TLS_CERT_PATH", "/etc/tls/cert.pem")]);
        let lookup = lookup_from(&map);

        let err = Config::from_env_with(lookup).expect_err("err");
        assert!(matches!(err, AppError::ConfigLoadError(_)));
    }

    #[test]
    fn from_env_errors_when_only_tls_key_set() {
        let map = HashMap::from([("TLS_KEY_PATH", "/etc/tls/key.pem")]);
        let lookup = lookup_from(&map);

        let err = Config::from_env_with(lookup).expect_err("err");
        assert!(matches!(err, AppError::ConfigLoadError(_)));
    }
}
