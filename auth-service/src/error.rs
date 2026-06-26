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
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Config error: {0}")]
    Config(String),
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

    #[test]
    fn from_io_error() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "gone");
        let err: AuthError = io_err.into();
        assert!(matches!(err, AuthError::Io(_)));
        assert!(format!("{err}").contains("gone"));
    }
}
