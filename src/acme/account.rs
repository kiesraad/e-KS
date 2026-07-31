//! ACME account handling. The account is created once per environment with
//! the `create_acme_account` tool (development crate) and deployed as the
//! `ACME_ACCOUNT_CREDENTIALS` secret; the application never registers
//! accounts on its own.

use std::path::Path;

use instant_acme::{Account, AccountBuilder, AccountCredentials, NewAccount};
use secrecy::ExposeSecret;

use crate::{AcmeConfig, AppError};

fn builder(root_ca_path: Option<&Path>) -> Result<AccountBuilder, AppError> {
    Ok(match root_ca_path {
        Some(path) => Account::builder_with_root(path)?,
        None => Account::builder()?,
    })
}

/// Parse the configured credentials; called at startup to fail fast and by
/// the renewer on every renewal.
pub fn parse_acme_account_credentials(acme: &AcmeConfig) -> Result<AccountCredentials, AppError> {
    serde_json::from_str(acme.account_credentials.expose_secret())
        .map_err(|e| AppError::ConfigLoadError(format!("ACME_ACCOUNT_CREDENTIALS is invalid: {e}")))
}

/// Restore the configured account at the directory.
pub(super) async fn load_account(acme: &AcmeConfig) -> Result<Account, AppError> {
    let credentials = parse_acme_account_credentials(acme)?;
    Ok(builder(acme.root_ca_path.as_deref())?
        .from_credentials(credentials)
        .await?)
}

/// Register a new account and return its credentials JSON; backs the
/// `create_acme_account` tool.
pub async fn create_acme_account(
    directory_url: String,
    contact: Option<&str>,
    root_ca_path: Option<&Path>,
) -> Result<String, AppError> {
    let contact: Vec<&str> = contact.into_iter().collect();
    let (_account, credentials) = builder(root_ca_path)?
        .create(
            &NewAccount {
                contact: &contact,
                terms_of_service_agreed: true,
                only_return_existing: false,
            },
            directory_url,
            None,
        )
        .await?;

    serde_json::to_string(&credentials).map_err(|e| AppError::AcmeError(e.into()))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn config(credentials: &str) -> AcmeConfig {
        AcmeConfig {
            directory_url: "https://acme.example/dir".to_string(),
            domain: "eks.example.nl".to_string(),
            account_credentials: secrecy::SecretString::from(credentials),
            root_ca_path: None,
        }
    }

    #[test]
    fn parse_rejects_invalid_credentials() {
        let Err(err) = parse_acme_account_credentials(&config("not json")) else {
            panic!("invalid JSON must not parse");
        };
        assert!(matches!(err, AppError::ConfigLoadError(_)));
        assert!(err.to_string().contains("ACME_ACCOUNT_CREDENTIALS"));

        // Valid JSON, but not the credentials shape.
        assert!(parse_acme_account_credentials(&config("{}")).is_err());
    }

    #[test]
    fn parse_accepts_the_credentials_shape() {
        let credentials = r#"{
            "id": "https://acme.example/acct/1",
            "key_pkcs8": "AAAA",
            "directory": "https://acme.example/dir"
        }"#;
        let _credentials =
            parse_acme_account_credentials(&config(credentials)).expect("valid credentials");
    }

    #[tokio::test]
    async fn load_account_fails_fast_on_invalid_credentials() {
        // Parsing happens before any network traffic.
        let Err(err) = load_account(&config("{}")).await else {
            panic!("invalid credentials must not load");
        };
        assert!(matches!(err, AppError::ConfigLoadError(_)));
    }

    #[test]
    fn builder_rejects_a_missing_root_ca() {
        assert!(builder(Some(Path::new("/nonexistent/root-ca.pem"))).is_err());
        builder(None).expect("default builder");
    }

    #[tokio::test]
    async fn create_account_rejects_a_missing_root_ca() {
        // The root CA is read before any network traffic.
        let err = create_acme_account(
            "https://acme.example/dir".to_string(),
            Some("mailto:beheer@example.nl"),
            Some(Path::new("/nonexistent/root-ca.pem")),
        )
        .await
        .expect_err("err");
        assert!(matches!(err, AppError::AcmeError(_)));
    }
}
