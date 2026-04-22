//! Per-stream AES-256-GCM encryption for event payloads.
//!
//! Each stream gets its own encryption key derived from a master secret
//! and the stream ID using HKDF-SHA256. Events are serialized with postcard,
//! then encrypted with AES-256-GCM using a random nonce prepended to the ciphertext.
//!
//! We encrypt event payloads at rest in the database, and decrypt them on demand when
//! fetching events. This provides confidentiality even if the database is compromised.
//!
//! This is not end-to-end encryption, since the server still has access to the plaintext
//! and performs encryption and decryption. The main threat model is an attacker who gains
//! read access to the database but not the server's memory or master secret.
//!
//! Note that this is a security-in-depth measure. The database should still be secure
//! and stored on a encrypted volume.

use aes_gcm::{
    AeadCore, AeadInPlace, Aes256Gcm, KeyInit,
    aead::{Nonce, OsRng},
};
use hkdf::Hkdf;
use secrecy::{ExposeSecret, SecretString};
use serde::{Serialize, de::DeserializeOwned};
use sha2::Sha256;
use uuid::Uuid;

use crate::{AppError, ElectionConfig};

const HKDF_SALT: &[u8] = b"e-KS event encryption v1";
const EVENT_KEY_INFO_PREFIX: &[u8] = b"event-key:";

/// Derives per-(stream, election) encryption keys from a master secret.
///
/// Holds a pre-extracted HKDF PRK computed once at startup.
/// Each call to [`derive_cipher`](EventEncryption::derive_cipher) runs only
/// the cheaper HKDF-Expand step.
///
/// Because a single `stream_id` covers all of a user's elections, the
/// `election` is mixed into the info string so events from different
/// elections retain independent keys.
#[derive(Clone)]
pub struct EventEncryption {
    hk: Hkdf<Sha256>,
}

impl EventEncryption {
    pub fn new(secret: &SecretString) -> Self {
        let hk = Hkdf::<Sha256>::new(Some(HKDF_SALT), secret.expose_secret().as_bytes());
        Self { hk }
    }

    /// Derive an [`EventCipher`] for the given (stream, election) pair.
    pub fn derive_cipher(&self, stream_id: Uuid, election: ElectionConfig) -> EventCipher {
        let election_id = election.stable_id();
        let info: Vec<u8> = EVENT_KEY_INFO_PREFIX
            .iter()
            .chain(stream_id.as_bytes())
            .chain(b":")
            .chain(election_id.as_bytes())
            .copied()
            .collect();

        let mut key = [0u8; 32];
        self.hk
            .expand(&info, &mut key)
            .expect("32 bytes is within HKDF-SHA256 output limit");

        let cipher =
            Aes256Gcm::new_from_slice(&key).expect("32 bytes is a valid AES-256 key length");

        EventCipher { cipher }
    }
}

impl std::fmt::Debug for EventEncryption {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("EventEncryption([REDACTED])")
    }
}

/// AES-256-GCM cipher for a single stream.
///
/// Encrypts and decrypts event payloads serialized with postcard.
/// Each ciphertext is prefixed with a random 12-byte nonce.
#[derive(Clone)]
pub struct EventCipher {
    cipher: Aes256Gcm,
}

const NONCE_LEN: usize = 12;

impl EventCipher {
    /// Serialize `event` with postcard and encrypt.
    ///
    /// Returns `nonce || ciphertext || tag`.
    pub fn encrypt<E: Serialize>(&self, event: &E) -> Result<Vec<u8>, AppError> {
        let nonce = Aes256Gcm::generate_nonce(&mut OsRng);

        let mut ciphertext = postcard::to_allocvec(event).map_err(|e| {
            AppError::ServerError(std::io::Error::new(std::io::ErrorKind::InvalidData, e))
        })?;

        self.cipher
            .encrypt_in_place(&nonce, b"", &mut ciphertext)
            .map_err(|e| AppError::ServerError(std::io::Error::other(e.to_string())))?;

        let mut out = Vec::with_capacity(NONCE_LEN + ciphertext.len());
        out.extend_from_slice(&nonce);
        out.extend_from_slice(&ciphertext);
        Ok(out)
    }

    /// Decrypt and deserialize an event payload from borrowed data.
    ///
    /// Expects `nonce (12 bytes) || ciphertext || tag`.
    pub fn decrypt<E: DeserializeOwned>(&self, data: &[u8]) -> Result<E, AppError> {
        self.decrypt_owned(data.to_vec())
    }

    /// Decrypt and deserialize from an already-owned buffer, avoiding a copy.
    ///
    /// Expects `nonce (12 bytes) || ciphertext || tag`.
    pub fn decrypt_owned<E: DeserializeOwned>(&self, mut data: Vec<u8>) -> Result<E, AppError> {
        if data.len() < NONCE_LEN {
            return Err(AppError::EventDecodeError(
                "ciphertext too short for nonce".to_string(),
            ));
        }

        let nonce = *Nonce::<Aes256Gcm>::from_slice(&data[..NONCE_LEN]);

        // Remove the nonce prefix so the Vec contains only ciphertext + tag,
        // then decrypt in place — this truncates the tag, leaving plaintext.
        let _ = data.drain(..NONCE_LEN);
        self.cipher
            .decrypt_in_place(&nonce, b"", &mut data)
            .map_err(|e| AppError::EventDecodeError(format!("AES-GCM decrypt failed: {e}")))?;

        postcard::from_bytes(&data)
            .map_err(|e| AppError::EventDecodeError(format!("payload deserialize failed: {e}")))
    }
}

impl std::fmt::Debug for EventCipher {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("EventCipher([REDACTED])")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_secret() -> SecretString {
        SecretString::from("test-encryption-secret")
    }

    const TEST_ELECTION: ElectionConfig = ElectionConfig::EK27;

    #[test]
    fn round_trip_encrypt_decrypt() {
        let enc = EventEncryption::new(&test_secret());
        let cipher = enc.derive_cipher(Uuid::new_v4(), TEST_ELECTION);

        let original: Vec<u8> = vec![1, 2, 3, 4, 5];
        let encrypted = cipher.encrypt(&original).unwrap();
        let decrypted: Vec<u8> = cipher.decrypt(&encrypted).unwrap();

        assert_eq!(original, decrypted);
    }

    #[test]
    fn different_streams_produce_different_ciphertexts() {
        let enc = EventEncryption::new(&test_secret());
        let cipher_a = enc.derive_cipher(Uuid::new_v4(), TEST_ELECTION);
        let cipher_b = enc.derive_cipher(Uuid::new_v4(), TEST_ELECTION);

        let data = "same payload";
        let enc_a = cipher_a.encrypt(&data).unwrap();
        let enc_b = cipher_b.encrypt(&data).unwrap();

        // Different keys → different ciphertext (with overwhelming probability)
        assert_ne!(enc_a, enc_b);
    }

    #[test]
    fn wrong_stream_fails_decryption() {
        let enc = EventEncryption::new(&test_secret());
        let cipher_a = enc.derive_cipher(Uuid::new_v4(), TEST_ELECTION);
        let cipher_b = enc.derive_cipher(Uuid::new_v4(), TEST_ELECTION);

        let encrypted = cipher_a.encrypt(&42u32).unwrap();
        let result = cipher_b.decrypt::<u32>(&encrypted);

        assert!(result.is_err());
    }

    #[test]
    fn different_elections_fail_decryption() {
        let enc = EventEncryption::new(&test_secret());
        let stream_id = Uuid::new_v4();
        let cipher_ek = enc.derive_cipher(stream_id, ElectionConfig::EK27);
        let cipher_ps = enc.derive_cipher(stream_id, ElectionConfig::PS27(crate::Province::GR));

        let encrypted = cipher_ek.encrypt(&42u32).unwrap();
        let result = cipher_ps.decrypt::<u32>(&encrypted);

        assert!(result.is_err());
    }

    #[test]
    fn wrong_secret_fails_decryption() {
        let stream_id = Uuid::new_v4();
        let cipher_a = EventEncryption::new(&test_secret()).derive_cipher(stream_id, TEST_ELECTION);
        let cipher_b = EventEncryption::new(&SecretString::from("different-secret"))
            .derive_cipher(stream_id, TEST_ELECTION);

        let encrypted = cipher_a.encrypt(&42u32).unwrap();
        let result = cipher_b.decrypt::<u32>(&encrypted);

        assert!(result.is_err());
    }

    #[test]
    fn tampered_ciphertext_fails() {
        let enc = EventEncryption::new(&test_secret());
        let cipher = enc.derive_cipher(Uuid::new_v4(), TEST_ELECTION);

        let mut encrypted = cipher.encrypt(&"hello").unwrap();
        // Flip a bit in the ciphertext (after the nonce)
        let last = encrypted.len() - 1;
        encrypted[last] ^= 0x01;

        let result = cipher.decrypt::<String>(&encrypted);
        assert!(result.is_err());
    }

    #[test]
    fn decrypt_too_short_data_fails() {
        let enc = EventEncryption::new(&test_secret());
        let cipher = enc.derive_cipher(Uuid::new_v4(), TEST_ELECTION);

        let result = cipher.decrypt::<u32>(&[0u8; 5]);
        assert!(result.is_err());
    }

    #[test]
    fn same_plaintext_produces_different_ciphertexts() {
        let enc = EventEncryption::new(&test_secret());
        let cipher = enc.derive_cipher(Uuid::new_v4(), TEST_ELECTION);

        let data = "repeated";
        let enc1 = cipher.encrypt(&data).unwrap();
        let enc2 = cipher.encrypt(&data).unwrap();

        // Random nonces → different ciphertexts
        assert_ne!(enc1, enc2);

        // Both decrypt to the same value
        let dec1: String = cipher.decrypt(&enc1).unwrap();
        let dec2: String = cipher.decrypt(&enc2).unwrap();
        assert_eq!(dec1, dec2);
        assert_eq!(dec1, "repeated");
    }
}
