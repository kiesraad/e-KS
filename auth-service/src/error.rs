use thiserror::Error;

pub type Result<T> = std::result::Result<T, AuthError>;

#[derive(Debug, Error)]
pub enum AuthError {
    #[error("XML error: {0}")]
    Xml(String),
    #[error("Crypto error: {0}")]
    Crypto(String),
    #[error("HTTP error: {0}")]
    Http(String),
    #[error("Config error: {0}")]
    Config(String),
    #[error("Template error: {0}")]
    Template(String),
}

impl From<askama::Error> for AuthError {
    fn from(e: askama::Error) -> Self {
        AuthError::Template(e.to_string())
    }
}

impl From<crate::saml::xml_parser::XmlError> for AuthError {
    fn from(e: crate::saml::xml_parser::XmlError) -> Self {
        AuthError::Xml(e.to_string())
    }
}

impl From<reqwest::Error> for AuthError {
    fn from(e: reqwest::Error) -> Self {
        AuthError::Http(e.to_string())
    }
}

/// A value (e.g. a metadata-derived URL) is not representable in an HTTP
/// header. `Config` rather than `Http`: it means the loaded metadata/config is
/// bad, not that a transient transport failure occurred.
impl From<axum::http::header::InvalidHeaderValue> for AuthError {
    fn from(e: axum::http::header::InvalidHeaderValue) -> Self {
        AuthError::Config(format!("value is not representable in an HTTP header: {e}"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display_xml_error() {
        let err = AuthError::Xml("bad tag".to_string());
        assert_eq!(format!("{err}"), "XML error: bad tag");
    }

    #[test]
    fn display_crypto_error() {
        let err = AuthError::Crypto("key failed".to_string());
        assert_eq!(format!("{err}"), "Crypto error: key failed");
    }

    #[test]
    fn display_http_error() {
        let err = AuthError::Http("timeout".to_string());
        assert_eq!(format!("{err}"), "HTTP error: timeout");
    }

    #[test]
    fn display_config_error() {
        let err = AuthError::Config("missing var".to_string());
        assert_eq!(format!("{err}"), "Config error: missing var");
    }
}
