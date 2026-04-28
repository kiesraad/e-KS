//! Loads runtime configuration from environment variables for AppState.
//! Used by AppState::new to construct service URLs and storage settings.
//!
//! For each configuration name `<NAME>`, an operator may set `<NAME>_FILE`
//! pointing at a file whose contents are the value.

use std::env;

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

    pub(super) const BAG_SERVICE_URL: &str = "http://localhost:8090";

    pub(super) const ID_DERIVATION_KEY: &str = "eks-dev-id-derivation-key-not-for-production";

    pub(super) const DEFAULT_ENCRYPTION_DERIVATION_KEY: &str =
        "eks-dev-encryption-derivation-key-not-for-production";

    pub(super) fn lookup(name: &str) -> Result<String, std::env::VarError> {
        std::collections::HashMap::from([
            ("STORAGE_URL", STORAGE_URL),
            #[cfg(not(feature = "embed-typst"))]
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
        .ok_or(std::env::VarError::NotPresent)
    }
}

/// Runtime configuration loaded from environment variables.
#[derive(Debug, Clone)]
pub struct Config {
    pub storage_url: String,
    #[cfg(not(feature = "embed-typst"))]
    pub typst_url: String,
    pub id_derivation_key: SecretString,
    pub encryption_derivation_key: SecretString,
}

/// Sync env-only lookup for non-secret configuration. Does not support the
/// `<NAME>_FILE` indirection — for secrets, go through [`Config::from_env`],
/// which uses `tokio::fs` to avoid blocking the runtime.
pub fn get_env(name: &'static str) -> Result<String, AppError> {
    match env::var(name) {
        Ok(value) => Ok(value),
        Err(_) => {
            #[cfg(feature = "dev-features")]
            {
                dev_defaults::lookup(name).map_err(|_| AppError::MissingEnvVar(name))
            }
            #[cfg(not(feature = "dev-features"))]
            {
                Err(AppError::MissingEnvVar(name))
            }
        }
    }
}

async fn read_secret_file(path: &str) -> Result<String, AppError> {
    let contents = tokio::fs::read_to_string(path)
        .await
        .map_err(|err| AppError::ConfigLoadError(format!("reading secret file {path}: {err}")))?;
    Ok(contents.trim().to_string())
}

async fn get_secret_with<L>(name: &'static str, lookup: &mut L) -> Result<String, AppError>
where
    L: FnMut(&str) -> Result<String, env::VarError>,
{
    let file_var = format!("{name}_FILE");
    let from_file = lookup(&file_var).ok();
    let from_env = lookup(name).ok();

    match (from_file, from_env) {
        (Some(_), Some(_)) => Err(AppError::ConfigLoadError(format!(
            "both {name} and {name}_FILE are set; choose one"
        ))),
        (Some(path), None) => read_secret_file(&path).await,
        (None, Some(value)) => Ok(value),
        (None, None) => {
            #[cfg(feature = "dev-features")]
            {
                dev_defaults::lookup(name).map_err(|_| AppError::MissingEnvVar(name))
            }
            #[cfg(not(feature = "dev-features"))]
            {
                Err(AppError::MissingEnvVar(name))
            }
        }
    }
}

impl Config {
    pub async fn from_env() -> Result<Self, AppError> {
        Self::from_env_with(|key: &str| env::var(key)).await
    }

    async fn from_env_with<L>(mut lookup: L) -> Result<Self, AppError>
    where
        L: FnMut(&str) -> Result<String, env::VarError>,
    {
        let storage_url = get_secret_with("STORAGE_URL", &mut lookup).await?;
        #[cfg(not(feature = "embed-typst"))]
        let typst_url = get_secret_with("TYPST_URL", &mut lookup).await?;
        let id_derivation_key = get_secret_with("ID_DERIVATION_KEY", &mut lookup).await?;

        let encryption_derivation_key =
            get_secret_with("ENCRYPTION_DERIVATION_KEY", &mut lookup).await?;

        Ok(Self {
            storage_url,
            #[cfg(not(feature = "embed-typst"))]
            typst_url,
            id_derivation_key: SecretString::from(id_derivation_key),
            encryption_derivation_key: SecretString::from(encryption_derivation_key),
        })
    }

    #[cfg(test)]
    pub fn new_test() -> Self {
        Self {
            storage_url: "memory://".to_string(),
            #[cfg(not(feature = "embed-typst"))]
            typst_url: "http://localhost:8080".to_string(),
            id_derivation_key: SecretString::from("test-secret-123"),
            encryption_derivation_key: SecretString::from("test-encryption-secret-123"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{collections::HashMap, path::PathBuf};

    fn lookup_from(
        map: &HashMap<&'static str, String>,
    ) -> impl FnMut(&str) -> Result<String, env::VarError> {
        let owned: HashMap<String, String> = map
            .iter()
            .map(|(k, v)| ((*k).to_string(), v.clone()))
            .collect();
        move |key| owned.get(key).cloned().ok_or(env::VarError::NotPresent)
    }

    /// Writes `contents` to a uniquely-named temp file and returns its path.
    /// Caller is responsible for cleanup via `cleanup`.
    fn temp_secret(label: &str, contents: &str) -> PathBuf {
        let path = std::env::temp_dir().join(format!(
            "eks-config-test-{}-{}-{}.secret",
            label,
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::write(&path, contents).expect("write temp secret");
        path
    }

    fn cleanup(path: &PathBuf) {
        let _ = std::fs::remove_file(path);
    }

    #[tokio::test]
    async fn get_secret_returns_env_value_when_set() {
        let map = HashMap::from([("TEST_CONFIG_ENV", "present".to_string())]);
        let mut lookup = lookup_from(&map);

        let value = get_secret_with("TEST_CONFIG_ENV", &mut lookup)
            .await
            .expect("env value");

        assert_eq!(value, "present");
    }

    #[tokio::test]
    async fn get_secret_reads_from_file_when_file_var_set() {
        let path = temp_secret("file-var", "from-disk");
        let map = HashMap::from([("TEST_SECRET_FILE", path.to_str().unwrap().to_string())]);
        let mut lookup = lookup_from(&map);

        let value = get_secret_with("TEST_SECRET", &mut lookup)
            .await
            .expect("file value");

        assert_eq!(value, "from-disk");
        cleanup(&path);
    }

    #[tokio::test]
    async fn get_secret_rejects_when_both_env_and_file_set() {
        let path = temp_secret("ambiguous", "from-disk");
        let map = HashMap::from([
            ("TEST_SECRET", "from-env".to_string()),
            ("TEST_SECRET_FILE", path.to_str().unwrap().to_string()),
        ]);
        let mut lookup = lookup_from(&map);

        let err = get_secret_with("TEST_SECRET", &mut lookup)
            .await
            .expect_err("ambiguous");

        match err {
            AppError::ConfigLoadError(msg) => {
                assert!(msg.contains("TEST_SECRET"));
                assert!(msg.contains("TEST_SECRET_FILE"));
            }
            other => panic!("expected ConfigLoadError, got {other:?}"),
        }
        cleanup(&path);
    }

    #[tokio::test]
    async fn get_secret_propagates_file_read_errors() {
        let map = HashMap::from([(
            "TEST_SECRET_FILE",
            "/definitely/does/not/exist/eks-secret".to_string(),
        )]);
        let mut lookup = lookup_from(&map);

        let err = get_secret_with("TEST_SECRET", &mut lookup)
            .await
            .expect_err("read failure");

        assert!(matches!(err, AppError::ConfigLoadError(_)));
    }

    #[tokio::test]
    async fn read_secret_file_strips_surrounding_whitespace() {
        let path = temp_secret("ws", "  \t\r\nvalue-with-whitespace\r\n  ");

        let value = read_secret_file(path.to_str().unwrap())
            .await
            .expect("read");

        assert_eq!(value, "value-with-whitespace");
        cleanup(&path);
    }

    #[cfg(not(feature = "embed-typst"))]
    #[tokio::test]
    async fn from_env_uses_env_values() {
        let map = HashMap::from([
            ("STORAGE_URL", "memory://test".to_string()),
            ("TYPST_URL", "http://typst.test".to_string()),
            ("ID_DERIVATION_KEY", "test-secret-123".to_string()),
            (
                "ENCRYPTION_DERIVATION_KEY",
                "test-encryption-secret-123".to_string(),
            ),
        ]);
        let lookup = lookup_from(&map);

        let config = Config::from_env_with(lookup).await.expect("config");

        assert_eq!(config.storage_url, "memory://test");
        assert_eq!(config.typst_url, "http://typst.test");
    }

    #[cfg(feature = "embed-typst")]
    #[tokio::test]
    async fn from_env_uses_env_values_with_embed_typst() {
        let map = HashMap::from([
            ("STORAGE_URL", "memory://test".to_string()),
            ("ID_DERIVATION_KEY", "test-secret-123".to_string()),
            (
                "ENCRYPTION_DERIVATION_KEY",
                "test-encryption-secret-123".to_string(),
            ),
        ]);
        let lookup = lookup_from(&map);

        let config = Config::from_env_with(lookup).await.expect("config");

        assert_eq!(config.storage_url, "memory://test");
    }

    #[cfg(not(feature = "embed-typst"))]
    #[tokio::test]
    async fn from_env_reads_secrets_from_files() {
        let storage = temp_secret("storage", "memory://from-file");
        let id = temp_secret("id", "id-key-from-file");
        let enc = temp_secret("enc", "enc-key-from-file");

        let map = HashMap::from([
            ("STORAGE_URL_FILE", storage.to_str().unwrap().to_string()),
            ("TYPST_URL", "http://typst.test".to_string()),
            ("ID_DERIVATION_KEY_FILE", id.to_str().unwrap().to_string()),
            (
                "ENCRYPTION_DERIVATION_KEY_FILE",
                enc.to_str().unwrap().to_string(),
            ),
        ]);

        let config = Config::from_env_with(lookup_from(&map))
            .await
            .expect("config from files");

        assert_eq!(config.storage_url, "memory://from-file");
        assert_eq!(config.typst_url, "http://typst.test");

        cleanup(&storage);
        cleanup(&id);
        cleanup(&enc);
    }

    #[cfg(feature = "dev-features")]
    #[tokio::test]
    async fn get_secret_returns_default_when_missing_in_dev_features() {
        let map = HashMap::new();
        let mut lookup = lookup_from(&map);

        let value = get_secret_with("ID_DERIVATION_KEY", &mut lookup)
            .await
            .expect("dev default");

        assert_eq!(value, dev_defaults::ID_DERIVATION_KEY);
    }

    #[cfg(not(feature = "dev-features"))]
    #[tokio::test]
    async fn get_secret_errors_when_missing_without_dev_features() {
        let map = HashMap::new();
        let mut lookup = lookup_from(&map);

        let err = get_secret_with("TEST_CONFIG_MISSING", &mut lookup)
            .await
            .expect_err("missing env");

        assert_eq!(
            err.to_string(),
            AppError::MissingEnvVar("TEST_CONFIG_MISSING").to_string()
        );
    }

    #[cfg(all(feature = "dev-features", not(feature = "embed-typst")))]
    #[tokio::test]
    async fn from_env_uses_defaults_in_dev_features() {
        let map = HashMap::new();
        let lookup = lookup_from(&map);

        let config = Config::from_env_with(lookup).await.expect("dev defaults");

        assert_eq!(config.storage_url, dev_defaults::STORAGE_URL);
        assert_eq!(config.typst_url, dev_defaults::TYPST_URL);
    }

    #[cfg(all(feature = "dev-features", feature = "embed-typst"))]
    #[tokio::test]
    async fn from_env_uses_defaults_in_dev_features() {
        let map = HashMap::new();
        let lookup = lookup_from(&map);

        let config = Config::from_env_with(lookup).await.expect("dev defaults");

        assert_eq!(config.storage_url, dev_defaults::STORAGE_URL);
    }

    #[cfg(not(feature = "dev-features"))]
    #[tokio::test]
    async fn from_env_errors_when_storage_missing_without_dev_features() {
        let map = HashMap::new();
        let lookup = lookup_from(&map);

        let err = Config::from_env_with(lookup)
            .await
            .expect_err("missing storage");

        assert_eq!(
            err.to_string(),
            AppError::MissingEnvVar("STORAGE_URL").to_string()
        );
    }
}
