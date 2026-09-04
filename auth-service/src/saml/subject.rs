//! The authenticated subject identifier handed to the embedding application.

use secrecy::SecretString;

/// A decrypted eID SubjectID (eID §7.6.3.4): the identifier plus the qualifier
/// saying which kind of identifier it is.
///
/// Produced by [`saml::validation`](crate::saml::validation) from an
/// `EncryptedID` and passed to
/// [`AuthState::on_authenticated`](crate::AuthState::on_authenticated).
#[derive(Debug, Clone)]
pub struct SubjectId {
    /// The subject identifier (BSN / pseudonym, PII). Wrapped so it never
    /// reaches `Debug`/logs and is zeroized on drop; call `.expose_secret()`
    /// to read it.
    pub value: SecretString,
    /// `@NameQualifier`: the eID identifier type, e.g.
    /// `urn:nl-eid-gdi:1.0:id:legacy-BSN` or `...:id:Pseudonym` (eID §7.6.3.4.4).
    pub name_qualifier: String,
}
