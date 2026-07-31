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

    pub(super) const ID_DERIVATION_KEY: &str = "eks-dev-id-derivation-key-not-for-production";

    pub(super) const DEFAULT_MASTER_ENCRYPTION_KEY: &str =
        "eks-dev-master-encryption-key-not-for-production";

    pub(super) fn lookup(name: &'static str) -> Result<String, std::env::VarError> {
        std::collections::HashMap::from([
            ("STORAGE_URL", STORAGE_URL),
            ("ID_DERIVATION_KEY", ID_DERIVATION_KEY),
            ("MASTER_ENCRYPTION_KEY", DEFAULT_MASTER_ENCRYPTION_KEY),
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

/// ACME (Let's Encrypt) certificate-renewal configuration.
#[derive(Debug, Clone)]
pub struct AcmeConfig {
    /// Deliberately no default, so production orders are always explicit.
    pub directory_url: String,
    /// FQDN to order the certificate for.
    pub domain: String,
    /// Account credentials JSON produced by the `create_acme_account` tool
    /// (development crate); contains the account's private key.
    pub account_credentials: SecretString,
    /// Extra trust root for the directory's own TLS (pebble testing only).
    pub root_ca_path: Option<PathBuf>,
}

/// Runtime configuration loaded from environment variables.
#[derive(Debug, Clone)]
pub struct Config {
    pub storage_url: SecretString,
    pub id_derivation_key: SecretString,
    pub master_encryption_key: SecretString,
    pub tls: Option<TlsConfig>,
    /// ACME certificate renewal; requires `tls`.
    pub acme: Option<AcmeConfig>,
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

/// Offline sanity check; the credentials are bound to their directory, so a
/// staging account can never be deployed against production (or vice versa).
fn check_account_credentials(credentials: &str, directory_url: &str) -> Result<(), AppError> {
    let value: serde_json::Value = serde_json::from_str(credentials).map_err(|e| {
        AppError::ConfigLoadError(format!("ACME_ACCOUNT_CREDENTIALS is not valid JSON: {e}"))
    })?;
    if value.get("directory").and_then(|v| v.as_str()) != Some(directory_url) {
        return Err(AppError::ConfigLoadError(
            "ACME_ACCOUNT_CREDENTIALS was not created for ACME_DIRECTORY_URL; \
             run `create_acme_account` against this directory"
                .to_string(),
        ));
    }
    Ok(())
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
        let id_derivation_key = get_env_with("ID_DERIVATION_KEY", &mut lookup)?;

        let master_encryption_key = get_env_with("MASTER_ENCRYPTION_KEY", &mut lookup)?;

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

        let acme = match (
            lookup("ACME_DIRECTORY_URL").ok().filter(|s| !s.is_empty()),
            lookup("ACME_DOMAIN").ok().filter(|s| !s.is_empty()),
        ) {
            (Some(directory_url), Some(domain)) => {
                if tls.is_none() {
                    return Err(AppError::ConfigLoadError(
                        "ACME renewal requires TLS_CERT_PATH and TLS_KEY_PATH".to_string(),
                    ));
                }
                let account_credentials = lookup("ACME_ACCOUNT_CREDENTIALS")
                    .ok()
                    .filter(|s| !s.is_empty())
                    .ok_or(AppError::MissingEnvVar("ACME_ACCOUNT_CREDENTIALS"))?;
                check_account_credentials(&account_credentials, &directory_url)?;
                Some(AcmeConfig {
                    directory_url,
                    domain,
                    account_credentials: SecretString::from(account_credentials),
                    root_ca_path: lookup("ACME_ROOT_CA_PATH")
                        .ok()
                        .filter(|s| !s.is_empty())
                        .map(PathBuf::from),
                })
            }
            (None, None) => None,
            _ => {
                return Err(AppError::ConfigLoadError(
                    "ACME_DIRECTORY_URL and ACME_DOMAIN must both be set, or both unset"
                        .to_string(),
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
            id_derivation_key: SecretString::from(id_derivation_key),
            master_encryption_key: SecretString::from(master_encryption_key),
            tls,
            acme,
            server_name,
            eks_key,
            disable_auth_service,
        })
    }

    #[cfg(test)]
    pub fn new_test() -> Self {
        Self {
            storage_url: SecretString::from("memory://"),
            id_derivation_key: SecretString::from("test-secret-123"),
            master_encryption_key: SecretString::from("test-encryption-secret-123"),
            tls: None,
            acme: None,
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

    #[test]
    fn from_env_uses_env_values() {
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

    #[cfg(feature = "dev-features")]
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

    #[test]
    fn from_env_returns_no_acme_when_unset() {
        let map = HashMap::new();
        let lookup = lookup_from(&map);

        let config = Config::from_env_with(lookup).expect("config");

        assert!(config.acme.is_none());
    }

    const TEST_ACME_CREDENTIALS: &str = concat!(
        r#"{"id":"https://acme-staging-v02.api.letsencrypt.org/acme/acct/123","#,
        r#""key_pkcs8":"bm90LWEtcmVhbC1rZXk","#,
        r#""directory":"https://acme-staging-v02.api.letsencrypt.org/directory"}"#
    );

    #[test]
    fn from_env_returns_acme_when_set_with_tls() {
        let map = HashMap::from([
            ("TLS_CERT_PATH", "/etc/tls/cert.pem"),
            ("TLS_KEY_PATH", "/etc/tls/key.pem"),
            (
                "ACME_DIRECTORY_URL",
                "https://acme-staging-v02.api.letsencrypt.org/directory",
            ),
            ("ACME_DOMAIN", "example.nl"),
            ("ACME_ACCOUNT_CREDENTIALS", TEST_ACME_CREDENTIALS),
        ]);
        let lookup = lookup_from(&map);

        let config = Config::from_env_with(lookup).expect("config");
        let acme = config.acme.expect("acme present");

        assert_eq!(
            acme.directory_url,
            "https://acme-staging-v02.api.letsencrypt.org/directory"
        );
        assert_eq!(acme.domain, "example.nl");
        assert_eq!(
            acme.account_credentials.expose_secret(),
            TEST_ACME_CREDENTIALS
        );
        assert!(acme.root_ca_path.is_none());
    }

    #[test]
    fn from_env_errors_when_acme_credentials_missing() {
        let map = HashMap::from([
            ("TLS_CERT_PATH", "/etc/tls/cert.pem"),
            ("TLS_KEY_PATH", "/etc/tls/key.pem"),
            ("ACME_DIRECTORY_URL", "https://localhost:14000/dir"),
            ("ACME_DOMAIN", "example.nl"),
        ]);
        let lookup = lookup_from(&map);

        let err = Config::from_env_with(lookup).expect_err("err");
        assert_eq!(
            err.to_string(),
            AppError::MissingEnvVar("ACME_ACCOUNT_CREDENTIALS").to_string()
        );
    }

    #[test]
    fn from_env_errors_when_acme_credentials_are_not_json() {
        let map = HashMap::from([
            ("TLS_CERT_PATH", "/etc/tls/cert.pem"),
            ("TLS_KEY_PATH", "/etc/tls/key.pem"),
            ("ACME_DIRECTORY_URL", "https://localhost:14000/dir"),
            ("ACME_DOMAIN", "example.nl"),
            ("ACME_ACCOUNT_CREDENTIALS", "not json"),
        ]);
        let lookup = lookup_from(&map);

        let err = Config::from_env_with(lookup).expect_err("err");
        assert!(matches!(err, AppError::ConfigLoadError(_)));
    }

    #[test]
    fn from_env_errors_when_acme_credentials_are_for_another_directory() {
        let map = HashMap::from([
            ("TLS_CERT_PATH", "/etc/tls/cert.pem"),
            ("TLS_KEY_PATH", "/etc/tls/key.pem"),
            ("ACME_DIRECTORY_URL", "https://localhost:14000/dir"),
            ("ACME_DOMAIN", "example.nl"),
            ("ACME_ACCOUNT_CREDENTIALS", TEST_ACME_CREDENTIALS),
        ]);
        let lookup = lookup_from(&map);

        let err = Config::from_env_with(lookup).expect_err("err");
        assert!(matches!(err, AppError::ConfigLoadError(_)));
    }

    #[test]
    fn from_env_errors_when_only_acme_directory_set() {
        let map = HashMap::from([
            ("TLS_CERT_PATH", "/etc/tls/cert.pem"),
            ("TLS_KEY_PATH", "/etc/tls/key.pem"),
            ("ACME_DIRECTORY_URL", "https://localhost:14000/dir"),
        ]);
        let lookup = lookup_from(&map);

        let err = Config::from_env_with(lookup).expect_err("err");
        assert!(matches!(err, AppError::ConfigLoadError(_)));
    }

    #[test]
    fn from_env_errors_when_acme_set_without_tls() {
        let map = HashMap::from([
            ("ACME_DIRECTORY_URL", "https://localhost:14000/dir"),
            ("ACME_DOMAIN", "example.nl"),
        ]);
        let lookup = lookup_from(&map);

        let err = Config::from_env_with(lookup).expect_err("err");
        assert!(matches!(err, AppError::ConfigLoadError(_)));
    }
}
