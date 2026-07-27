//! The `tvs-mock` deployment build: talk to the online (shared) TVS mock
//! service. It defaults `TVS_ENV` to `test` (so the RD endpoint resolves to the
//! TVS mock) and `BASE_URL` to `http://localhost:3000`, and embeds the committed
//! DV cert/key bundle into the binary, so the auth flow works on a host that has
//! no `CERTS_DIR` on disk (e.g. the branch test website). It does not run a TVS
//! back-channel itself.
//!
//! This embeds the committed test private keys into the binary: it is strictly
//! for mock deployments and must never be enabled for a real environment.
use std::{path::PathBuf, sync::OnceLock};

use crate::error::AuthError;

/// Default for a required env var when the variable is unset; an explicit
/// value still overrides this. `CERTS_DIR` is not
/// here because the bundle is embedded rather than read from a path (see
/// [`certs_dir`]).
pub(crate) fn default_for(name: &str) -> Option<String> {
    match name {
        "TVS_ENV" => Some("test".to_string()),
        "BASE_URL" => Some("http://localhost:3000".to_string()),
        _ => None,
    }
}

/// The committed DV cert/key bundle (`auth-service/fixtures`), embedded at
/// compile time. Paths are relative to this source file. Only the files the
/// DV side reads are embedded; the `rd-*` fixtures (for running an RD mock)
/// are not, since this build uses the online TVS mock service.
macro_rules! embedded_fixtures {
    ($($name:literal),* $(,)?) => {
        &[$(($name, include_bytes!(concat!("../fixtures/", $name)) as &[u8])),*]
    };
}

const FIXTURES: &[(&str, &[u8])] = embedded_fixtures![
    // mTLS client identity for the back-channel. (The test CA pinned for the
    // back-channel *server* is embedded separately and not read from a path;
    // see [`crate::saml::pki::BACKCHANNEL_ROOT_CA_PEM`].)
    "dv-tls.pem",
    "dv-tls-key.pem",
    // DV SAML signing keys (the second pair is optional, for rollover).
    "dv-signing-1.pem",
    "dv-signing-1-key.pem",
    "dv-signing-2.pem",
    "dv-signing-2-key.pem",
    // DV SAML encryption keys (the second pair is optional, for rollover).
    "dv-encryption-1.pem",
    "dv-encryption-1-key.pem",
    "dv-encryption-2.pem",
    "dv-encryption-2-key.pem",
];

/// Materialize the embedded DV bundle into a per-process temp directory the
/// first time it is needed and hand back its path, so the existing
/// path-based key loaders read it exactly like a real `CERTS_DIR`. The
/// extraction happens once per process (subsequent calls reuse the result).
pub(crate) fn certs_dir() -> Result<PathBuf, AuthError> {
    static DIR: OnceLock<Result<PathBuf, String>> = OnceLock::new();
    DIR.get_or_init(materialize)
        .clone()
        .map_err(AuthError::Config)
}

fn materialize() -> Result<PathBuf, String> {
    let dir = std::env::temp_dir().join(format!("eks-tvs-mock-{}", std::process::id()));
    std::fs::create_dir_all(&dir).map_err(|e| format!("create {}: {e}", dir.display()))?;
    for (name, bytes) in FIXTURES {
        let path = dir.join(name);
        std::fs::write(&path, bytes).map_err(|e| format!("write {}: {e}", path.display()))?;
    }
    tracing::warn!(
        certs_dir = %dir.display(),
        "tvs-mock: using embedded DV test keys against the online TVS mock (mock builds only, never a real deployment)"
    );
    Ok(dir)
}
