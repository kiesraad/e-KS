use std::fmt::{Display, Formatter};

use crate::AppError;

#[derive(Debug)]
pub enum SamlError {
    // Key/certificate loading
    KeyLoadError(Box<dyn std::error::Error + Send + Sync>),
    ServiceProviderBuildError(Box<dyn std::error::Error + Send + Sync>),
    InvalidCertificate(openssl::error::ErrorStack),
    InvalidBase64Certificate(base64::DecodeError),
    CertificateTrustError(rustls::Error),

    // IdP metadata validation
    IdpMetadataParseError(quick_xml::de::DeError),
    IdpMetadataLoadError(reqwest::Error),
    MissingValidUntil,
    IdpMetadataExpired,
    MissingIdpSsoDescriptor,
    InvalidIdpMetadataSignature,

    // Crypto
    CryptoError(samael::crypto::Error),
}

impl Display for SamlError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            SamlError::KeyLoadError(e) => {
                write!(f, "failed to load key or certificate from disk: {e}")
            }
            SamlError::ServiceProviderBuildError(e) => {
                write!(f, "failed to build SAML service provider: {e}")
            }
            SamlError::InvalidCertificate(e) => write!(f, "invalid X.509 certificate: {e}"),
            SamlError::InvalidBase64Certificate(e) => {
                write!(f, "certificate contains invalid base64: {e}")
            }
            SamlError::CertificateTrustError(e) => {
                write!(f, "certificate trust chain verification failed: {e}")
            }
            SamlError::IdpMetadataParseError(e) => {
                write!(f, "failed to parse IdP metadata XML: {e}")
            }
            SamlError::IdpMetadataLoadError(e) => write!(f, "failed to load IdP metadata: {e}"),
            SamlError::MissingValidUntil => {
                write!(f, "IdP metadata is missing a validUntil timestamp")
            }
            SamlError::IdpMetadataExpired => write!(f, "IdP metadata has expired"),
            SamlError::MissingIdpSsoDescriptor => {
                write!(f, "IdP metadata contains no SSO descriptor")
            }
            SamlError::InvalidIdpMetadataSignature => {
                write!(f, "IdP metadata signature is invalid")
            }
            SamlError::CryptoError(e) => write!(f, "cryptographic operation failed: {e}"),
        }
    }
}

impl std::error::Error for SamlError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            SamlError::KeyLoadError(e) => Some(e.as_ref()),
            SamlError::ServiceProviderBuildError(e) => Some(e.as_ref()),
            SamlError::InvalidCertificate(e) => Some(e),
            SamlError::InvalidBase64Certificate(e) => Some(e),
            SamlError::CertificateTrustError(e) => Some(e),
            SamlError::IdpMetadataParseError(e) => Some(e),
            SamlError::IdpMetadataLoadError(e) => Some(e),
            SamlError::CryptoError(e) => Some(e),
            _ => None,
        }
    }
}

impl From<SamlError> for AppError {
    fn from(_: SamlError) -> Self {
        AppError::InternalServerError
    }
}
