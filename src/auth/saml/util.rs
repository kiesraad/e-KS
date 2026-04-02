use base64::{Engine, prelude::BASE64_STANDARD};
use openssl::x509::X509;

use crate::auth::saml::SamlError;

pub fn string_to_der(certificate: &str) -> Result<X509, SamlError> {
    let certificate: String = certificate
        .chars()
        .filter(|c| !c.is_ascii_whitespace())
        .collect();

    let certificate = BASE64_STANDARD
        .decode(certificate)
        .map_err(SamlError::InvalidBase64Certificate)?;

    X509::from_der(&certificate).map_err(SamlError::InvalidCertificate)
}
