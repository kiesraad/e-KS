use crate::{
    config::TlsConfig,
    error::{AuthError, Result},
};
use base64::{Engine, engine::general_purpose::STANDARD as BASE64};
use secrecy::{ExposeSecret, SecretString};
use sha1::{Digest, Sha1};
use sha2::Sha256;
use std::{
    fs,
    path::{Path, PathBuf},
};
use tracing::{debug, warn};

#[derive(Debug, Clone)]
pub struct KeyPaths {
    pub cert: PathBuf,
    pub key: PathBuf,
}

#[derive(Debug, Clone)]
pub struct KeyPair {
    pub cert_pem: String,
    /// The private key in PEM. Wrapped so it is never printed via `Debug` and is
    /// zeroized on drop; call `.expose_secret()` to feed it to the signer.
    pub key_pem: SecretString,
    /// SHA-1 hex of the DER-encoded certificate (used to match KeyName in signatures).
    pub key_name: String,
    /// The certificate as base64 text, without PEM headers or whitespace.
    pub cert_base64: String,
}

impl KeyPair {
    /// Build a [`KeyPair`] from a certificate PEM, deriving `key_name` and
    /// `cert_base64` from it. Pass an empty `key_pem` for public-only
    /// certificates (metadata-advertised certs that never produce a signature).
    pub fn from_pem(cert_pem: String, key_pem: SecretString) -> Self {
        Self {
            key_name: derive_key_name(&cert_pem),
            cert_base64: cert_base64(&cert_pem),
            cert_pem,
            key_pem,
        }
    }

    /// Whether a signature's `<ds:KeyName>` identifies this certificate.
    ///
    /// The eID §7.6 message tables require a `<KeyInfo>` with a `<KeyName>` (or
    /// `<X509Certificate>`) but do not fix the KeyName format. The TVS
    /// Routeringsdienst references its certs by their SHA-1 thumbprint; the
    /// DigiD Authenticatiedienst signs the Assertion referencing the same kind
    /// of cert by its **SHA-256** thumbprint. Both forms identify the cert from
    /// verified metadata (a lookup key, not a security primitive), so accept
    /// either thumbprint algorithm.
    pub fn matches_key_name(&self, candidate: &str) -> bool {
        let candidate = candidate.trim();
        self.key_name == candidate
            || derive_key_names(&self.cert_pem)
                .iter()
                .any(|n| n == candidate)
    }
}

#[derive(Default, Debug, Clone)]
pub struct KeySet {
    pub signing: Vec<KeyPair>,
    pub encryption: Vec<KeyPair>,
}

impl KeySet {
    /// The key used to sign outgoing messages. [`load_key_set`] rejects an
    /// empty signing list, so this only panics on a hand-built `KeySet` (e.g.
    /// [`Default::default`]) that never loaded keys.
    pub fn primary_signing(&self) -> &KeyPair {
        self.signing
            .first()
            .expect("KeySet validated non-empty at load")
    }
}

pub fn cert_base64(cert_pem: &str) -> String {
    cert_pem
        .lines()
        .filter(|l| !l.starts_with("-----"))
        .collect::<String>()
        // Remove any whitespace remaining within lines.
        .chars()
        .filter(|c| !c.is_whitespace())
        .collect()
}

/// Wrap a base64-encoded DER certificate (e.g. the contents of a metadata
/// `<ds:X509Certificate>`) into PEM, with the canonical 64-character lines.
/// Inverse of [`cert_base64`].
pub fn pem_from_cert_base64(cert_b64: &str) -> String {
    let body: String = cert_b64.chars().filter(|c| !c.is_whitespace()).collect();
    let wrapped = body
        .as_bytes()
        .chunks(64)
        .map(|c| std::str::from_utf8(c).unwrap_or_default())
        .collect::<Vec<_>>()
        .join("\n");
    format!("-----BEGIN CERTIFICATE-----\n{wrapped}\n-----END CERTIFICATE-----")
}

/// Derive the `KeyName` identifier for a certificate.
///
/// Lowercase-hex SHA-1 thumbprint of the DER-encoded certificate: the
/// convention SAML metadata uses for `<ds:KeyName>` (and what the TVS
/// Routeringsdienst emits, both in its `KeyDescriptor`s and in the signature
/// over its metadata). Used purely as a lookup key to match a signature's
/// `KeyInfo` against a cert from verified metadata; not a security primitive,
/// so SHA-1 here is not a weakness.
pub fn derive_key_name(cert_pem: &str) -> String {
    hex(&Sha1::digest(cert_der(cert_pem)))
}

/// All `<ds:KeyName>` forms a peer might use to reference this certificate: the
/// lowercase-hex SHA-1 thumbprint (TVS RD convention) and the SHA-256 thumbprint
/// (DigiD AD convention). Used by [`KeyPair::matches_key_name`] to look a
/// signature's `KeyInfo` up against a cert from verified metadata regardless of
/// which thumbprint algorithm the signer chose (eID SAML: "KeyName MAY be any
/// string").
pub fn derive_key_names(cert_pem: &str) -> Vec<String> {
    let der = cert_der(cert_pem);
    vec![hex(&Sha1::digest(&der)), hex(&Sha256::digest(&der))]
}

/// The DER bytes of a PEM certificate (empty on malformed base64).
fn cert_der(cert_pem: &str) -> Vec<u8> {
    BASE64
        .decode(cert_base64(cert_pem).as_bytes())
        .unwrap_or_default()
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

fn load_key_pair(paths: &KeyPaths) -> Result<KeyPair> {
    // SECURITY: never log key_pem contents; only the public path it came from.
    debug!(
        "[keys] Loading key pair: cert={}, key={}",
        paths.cert.display(),
        paths.key.display()
    );
    let cert_pem = fs::read_to_string(&paths.cert).map_err(|e| {
        AuthError::Config(format!("Failed to read cert {}: {e}", paths.cert.display()))
    })?;
    let key_pem = fs::read_to_string(&paths.key).map_err(|e| {
        AuthError::Config(format!("Failed to read key {}: {e}", paths.key.display()))
    })?;

    let pair = KeyPair::from_pem(cert_pem, SecretString::from(key_pem));
    debug!(
        "[keys] Loaded key pair: key_name={} (cert={}, cert_len={}, key_len={})",
        pair.key_name,
        paths.cert.display(),
        pair.cert_pem.len(),
        pair.key_pem.expose_secret().len()
    );
    Ok(pair)
}

pub fn load_key_set(signing: &[KeyPaths], encryption: &[KeyPaths]) -> Result<KeySet> {
    debug!(
        "[keys] load_key_set: {} signing path(s), {} encryption path(s)",
        signing.len(),
        encryption.len()
    );
    if signing.is_empty() {
        return Err(AuthError::Config(
            "at least one signing key pair is required".to_string(),
        ));
    }
    let signing = signing
        .iter()
        .map(load_key_pair)
        .collect::<Result<Vec<_>>>()?;
    let encryption = encryption
        .iter()
        .map(load_key_pair)
        .collect::<Result<Vec<_>>>()?;
    debug!(
        "[keys] load_key_set OK: signing={}, encryption={}",
        signing.len(),
        encryption.len()
    );
    Ok(KeySet {
        signing,
        encryption,
    })
}

/// Load a certificate (public key only) into a [`KeyPair`] with an empty
/// `key_pem`. Used for certificates advertised in metadata but never used to
/// produce a signature (e.g. the DV's mTLS client certificate).
pub fn load_cert(cert_path: &Path) -> Result<KeyPair> {
    let cert_pem = fs::read_to_string(cert_path).map_err(|e| {
        AuthError::Config(format!("Failed to read cert {}: {e}", cert_path.display()))
    })?;

    Ok(KeyPair::from_pem(
        cert_pem,
        SecretString::from(String::new()),
    ))
}

/// Load the DV's mTLS client certificate so it can be published as an extra
/// `use="signing"` KeyDescriptor in the SP metadata. eID §8.3 requires the TLS
/// client certificate to be advertised as a signing certificate.
///
/// Best-effort: the mTLS handshake reads the certificate separately at request
/// time (see [`crate::bindings::soap`]), so a missing or unreadable cert here
/// is logged and omitted from the metadata rather than failing startup. An empty
/// path (an unconfigured [`TlsConfig`], e.g. in tests) is treated as "not
/// configured" without a warning.
pub fn load_metadata_tls_cert(cert_path: &Path) -> Option<KeyPair> {
    if cert_path.as_os_str().is_empty() {
        return None;
    }
    match load_cert(cert_path) {
        Ok(cert) => Some(cert),
        Err(e) => {
            warn!("[keys] TLS client cert not published in SP metadata: {e}");
            None
        }
    }
}

// ---------------------------------------------------------------------------
// On-disk bundle layout
// ---------------------------------------------------------------------------

// File layout of the DV cert/key bundle under `certs_dir`, defined once so every
// caller derives the same paths. The `tvs-mock` `FIXTURES` list mirrors these
// names (it needs literals for `include_bytes!`).
const TLS_CERT_FILE: &str = "dv-tls.pem";
const TLS_KEY_FILE: &str = "dv-tls-key.pem";
pub const SIGNING_BASES: &[&str] = &["dv-signing-1", "dv-signing-2"];
pub const ENCRYPTION_BASES: &[&str] = &["dv-encryption-1", "dv-encryption-2"];

/// The cert/key file paths for a key-family base name under `certs_dir`.
pub fn key_pair_paths(certs_dir: &Path, base: &str) -> KeyPaths {
    KeyPaths {
        cert: certs_dir.join(format!("{base}.pem")),
        key: certs_dir.join(format!("{base}-key.pem")),
    }
}

/// The mTLS client cert/key paths under `certs_dir` (eID §9.4 back-channel).
pub fn tls_paths(certs_dir: &Path) -> TlsConfig {
    TlsConfig {
        client_cert: certs_dir.join(TLS_CERT_FILE),
        client_key: certs_dir.join(TLS_KEY_FILE),
    }
}

/// Build the cert/key paths for the key families under `certs_dir`.
///
/// The first base's key pair is mandatory: the list always contains at least
/// one. Every base after it (e.g. a second key kept for rollover) is optional:
/// it is included only when its certificate file is present on disk, so a
/// single-key bundle publishes a single key pair in the metadata.
pub fn discover_key_paths(certs_dir: &Path, bases: &[&str]) -> Vec<KeyPaths> {
    bases
        .iter()
        .enumerate()
        .filter_map(|(index, base)| {
            let paths = key_pair_paths(certs_dir, base);
            if index > 0 && !paths.cert.exists() {
                return None;
            }
            Some(paths)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    const FAKE_PEM: &str = "\
-----BEGIN CERTIFICATE-----
MIIBkTCB+wIJALRiMLAh0WNHMA0GCSqGSIb3DQEBCwUAMBExDzANBgNVBAMMBn\
Rlc3RDQTAEFW0yNTAxMDEwMDAwMDBaFw0yNjAxMDEwMDAwMDBaMBExDzANBgNVBA\
MMBnRlc3RDQTBcMA0GCSqGSIb3DQEBAQUAA0sAMEgCQQC7o96P+5MhMjCnSGfn\
MhKxGdzQ7vNvPJGK7eCRvig6V7l6x2mBOFp2Z9gE4yrGS0ISjqRIG1WQ5rOb3U\
z9AgMBAAEwDQYJKoZIhvcNAQELBQADQQBb+u1uAt6HlG7MFQHtWJ0RI0U8C/XI\
fDqFa7OmGjPGqEjNdvTY3Zll8TfhUPGCNjBHPkTO1LjI/mO07m7bZO4\n\
-----END CERTIFICATE-----";

    #[test]
    fn cert_base64_strips_pem_headers() {
        let b64 = cert_base64(FAKE_PEM);
        assert!(!b64.contains("BEGIN"));
        assert!(!b64.contains("END"));
        assert!(!b64.contains('\n'));
        assert!(!b64.is_empty());
    }

    #[test]
    fn cert_base64_empty_input_returns_empty() {
        assert_eq!(cert_base64(""), "");
    }

    #[test]
    fn derive_key_name_is_hex_sha1() {
        let name = derive_key_name(FAKE_PEM);
        assert_eq!(name.len(), 40, "SHA-1 hex should be 40 chars");
        assert!(name.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn derive_key_name_is_deterministic() {
        let a = derive_key_name(FAKE_PEM);
        let b = derive_key_name(FAKE_PEM);
        assert_eq!(a, b);
    }

    #[test]
    fn derive_key_names_yields_sha1_and_sha256() {
        let names = derive_key_names(FAKE_PEM);
        assert_eq!(names.len(), 2);
        assert_eq!(names[0].len(), 40, "first is SHA-1 (40 hex chars)");
        assert_eq!(names[1].len(), 64, "second is SHA-256 (64 hex chars)");
        assert_eq!(names[0], derive_key_name(FAKE_PEM));
    }

    fn fake_key_pair() -> KeyPair {
        KeyPair::from_pem(FAKE_PEM.to_string(), String::new().into())
    }

    #[test]
    fn matches_key_name_accepts_sha1_thumbprint() {
        let kp = fake_key_pair();
        let sha1 = derive_key_names(FAKE_PEM)[0].clone();
        assert!(kp.matches_key_name(&sha1));
    }

    #[test]
    fn matches_key_name_accepts_sha256_thumbprint() {
        // The DigiD AD references its signing cert by SHA-256 thumbprint, not
        // the SHA-1 form stored in `key_name`; both must match.
        let kp = fake_key_pair();
        let sha256 = derive_key_names(FAKE_PEM)[1].clone();
        assert_ne!(sha256, kp.key_name);
        assert!(kp.matches_key_name(&sha256));
        assert!(kp.matches_key_name(&format!("  {sha256}  ")), "trims input");
    }

    #[test]
    fn matches_key_name_rejects_unknown() {
        let kp = fake_key_pair();
        assert!(!kp.matches_key_name("deadbeef"));
    }

    fn fixtures_dir() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("fixtures")
    }

    #[test]
    fn pem_from_cert_base64_round_trips_cert_base64() {
        // Wrapping the stripped base64 back into PEM and stripping it again is a
        // no-op, and the wrapped body uses the canonical 64-char lines.
        let b64 = cert_base64(FAKE_PEM);
        let pem = pem_from_cert_base64(&b64);
        assert!(pem.starts_with("-----BEGIN CERTIFICATE-----\n"));
        assert!(pem.ends_with("\n-----END CERTIFICATE-----"));
        assert_eq!(cert_base64(&pem), b64);
        for line in pem
            .lines()
            .filter(|l| !l.starts_with("-----") && !l.is_empty())
        {
            assert!(line.len() <= 64);
        }
    }

    #[test]
    fn load_key_set_errors_when_signing_empty() {
        // An empty signing list is a config error, so `primary_signing` is
        // guaranteed to succeed on any loaded KeySet.
        let err = load_key_set(&[], &[]).unwrap_err();
        assert!(
            matches!(&err, AuthError::Config(m) if m.contains("signing key pair")),
            "{err:?}"
        );
    }

    #[test]
    fn load_key_set_errors_when_cert_missing() {
        // A signing entry whose cert file does not exist fails with a Config error.
        let paths = KeyPaths {
            cert: fixtures_dir().join("does-not-exist.pem"),
            key: fixtures_dir().join("does-not-exist-key.pem"),
        };
        let err = load_key_set(std::slice::from_ref(&paths), &[]).unwrap_err();
        assert!(
            matches!(&err, AuthError::Config(m) if m.contains("Failed to read cert")),
            "{err:?}"
        );
    }

    #[test]
    fn load_key_set_errors_when_key_missing() {
        // Cert present, private key absent: the key-read branch reports the error.
        let paths = KeyPaths {
            cert: fixtures_dir().join("dv-signing-1.pem"),
            key: fixtures_dir().join("does-not-exist-key.pem"),
        };
        let err = load_key_set(std::slice::from_ref(&paths), &[]).unwrap_err();
        assert!(
            matches!(&err, AuthError::Config(m) if m.contains("Failed to read key")),
            "{err:?}"
        );
    }

    #[test]
    fn load_key_set_loads_fixture_pair() {
        // The success path: the committed DV signing fixture loads and derives a
        // key name and public base64.
        let paths = KeyPaths {
            cert: fixtures_dir().join("dv-signing-1.pem"),
            key: fixtures_dir().join("dv-signing-1-key.pem"),
        };
        let set = load_key_set(std::slice::from_ref(&paths), &[]).unwrap();
        assert_eq!(set.signing.len(), 1);
        assert_eq!(set.encryption.len(), 0);
        assert_eq!(set.signing[0].key_name.len(), 40);
        assert!(!set.signing[0].cert_base64.is_empty());
        assert!(!set.signing[0].key_pem.expose_secret().is_empty());
    }

    #[test]
    fn load_cert_reads_public_cert_without_private_key() {
        let cert = load_cert(&fixtures_dir().join("dv-tls.pem")).unwrap();
        assert_eq!(cert.key_name.len(), 40);
        assert!(!cert.cert_base64.is_empty());
        // A public-only cert carries no private key.
        assert!(cert.key_pem.expose_secret().is_empty());
    }

    #[test]
    fn load_cert_errors_when_file_missing() {
        let err = load_cert(&fixtures_dir().join("nope.pem")).unwrap_err();
        assert!(
            matches!(&err, AuthError::Config(m) if m.contains("Failed to read cert")),
            "{err:?}"
        );
    }
}
