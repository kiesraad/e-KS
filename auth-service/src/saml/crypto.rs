//! XML-DSig / XML-Enc backend adapter.
use crate::error::{AuthError, Result};
use bergshamra_dsig::DsigContext;
use bergshamra_enc::{EncContext, decrypt::decrypt as backend_decrypt};
use bergshamra_keys::{KeysManager, loader};
use tracing::warn;

/// Outcome of verifying an XML signature against a certificate.
pub enum SignatureVerification {
    Valid,
    /// The signature did not validate; carries the backend's reason.
    Invalid(String),
}

/// Sign `xml` in place with the RSA private key (`private_key_pem`).
///
/// `xml` must already contain an inline, unfilled `ds:Signature` template; the
/// backend computes and fills its `DigestValue` and `SignatureValue`.
pub fn sign(xml: &str, private_key_pem: &str) -> Result<String> {
    let key = loader::load_rsa_private_pem(private_key_pem.as_bytes())
        .map_err(|e| AuthError::Crypto(format!("Failed to load signing key: {e}")))?;
    let mut mgr = KeysManager::new();
    mgr.add_key(key);
    let ctx = DsigContext::new(mgr);
    bergshamra_dsig::sign::sign(&ctx, xml)
        .map_err(|e| AuthError::Crypto(format!("XML signing failed: {e}")))
}

/// Verify the signature anchored by `xml` against `cert_pem`, which is used as
/// the sole trusted key. `Err` indicates a cert-load or backend error; a failed
/// signature match is `Ok(Invalid)`.
pub fn verify_signature(xml: &str, cert_pem: &str) -> Result<SignatureVerification> {
    let key = loader::load_x509_cert_pem(cert_pem.as_bytes())
        .map_err(|e| AuthError::Crypto(format!("Failed to load verification cert: {e}")))?;
    let mut mgr = KeysManager::new();
    mgr.add_key(key);
    let ctx = DsigContext::new(mgr).with_trusted_keys_only(true);
    match bergshamra_dsig::verify::verify(&ctx, xml) {
        Ok(bergshamra_dsig::VerifyResult::Valid { .. }) => Ok(SignatureVerification::Valid),
        Ok(bergshamra_dsig::VerifyResult::Invalid { reason }) => {
            Ok(SignatureVerification::Invalid(reason))
        }
        Err(e) => Err(AuthError::Crypto(format!(
            "Signature verification error: {e}"
        ))),
    }
}

/// Decrypt an XML-Enc fragment, trying each `(private_key_pem, key_name)` pair
/// and returning the first success.
///
/// Key transport is crypto-bound to the keypair (the backend unwraps the
/// `EncryptedKey` with the RSA private key, not by matching `<KeyName>`), so
/// trying each key in turn decrypts a blob wrapped to any configured key and
/// keeps cert rollover working regardless of list order. `key_name` is for
/// diagnostics only; keys that fail to load are skipped.
pub fn decrypt(encrypted_xml: &str, keys: &[(&str, &str)]) -> Result<String> {
    let mut last_err: Option<String> = None;
    for (key_pem, key_name) in keys {
        // SECURITY: never log key_pem; key_name is a thumbprint of the public cert.
        let bkey = match loader::load_rsa_private_pem(key_pem.as_bytes()) {
            Ok(bkey) => bkey.with_name(*key_name),
            Err(e) => {
                warn!("[crypto] failed to load decryption key {key_name}: {e}");
                continue;
            }
        };
        let mut mgr = KeysManager::new();
        mgr.add_key(bkey);
        let ctx = EncContext::new(mgr);
        match backend_decrypt(&ctx, encrypted_xml) {
            Ok(xml) => return Ok(xml),
            Err(e) => last_err = Some(e.to_string()),
        }
    }
    Err(AuthError::Crypto(format!(
        "decryption failed: {}",
        last_err.unwrap_or_else(|| "no usable decryption key".to_string())
    )))
}
