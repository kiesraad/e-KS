//! Level of Assurance mapping and the DV minimum.

/// SAML AuthnContext Level of Assurance, per Koppelvlakspecificatie eID SAML
/// v4.4 §10.3. Variants are ordered by increasing assurance so `>=` directly
/// implements the eID §7.6.3.2 / TVS "Checklist Testen" v2.1 T6 rule:
/// "the DV MUST accept authentications with a level equal to or higher than
/// the minimum level registered for the Service."
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum LevelOfAssurance {
    /// Basis. URIs (eID SAML v4.4 §10.3):
    /// `urn:oasis:names:tc:SAML:2.0:ac:classes:PasswordProtectedTransport`,
    /// `http://eID.logius.nl/LoA/basic`.
    Basic,
    /// Midden / eIDAS Low. URIs (eID SAML v4.4 §10.3):
    /// `urn:oasis:names:tc:SAML:2.0:ac:classes:MobileTwoFactorContract`,
    /// `http://eidas.europa.eu/LoA/low`.
    Low,
    /// Substantieel / eIDAS Substantial. URIs (eID SAML v4.4 §10.3):
    /// `urn:oasis:names:tc:SAML:2.0:ac:classes:Smartcard`,
    /// `http://eidas.europa.eu/LoA/substantial`.
    Substantial,
    /// Hoog / eIDAS High. URIs (eID SAML v4.4 §10.3):
    /// `urn:oasis:names:tc:SAML:2.0:ac:classes:SmartcardPKI`,
    /// `http://eidas.europa.eu/LoA/high`.
    High,
}

impl LevelOfAssurance {
    /// Map an `AuthnContextClassRef` URI to its level, per the eID SAML v4.4
    /// §10.3 table. Each level has two accepted spellings: the SAML
    /// `ac:classes` URN and the eID/eIDAS URL the RD emits. Returns `None` for
    /// any URI not in the table.
    pub fn from_uri(uri: &str) -> Option<Self> {
        match uri {
            "urn:oasis:names:tc:SAML:2.0:ac:classes:PasswordProtectedTransport"
            | "http://eID.logius.nl/LoA/basic" => Some(Self::Basic),
            "urn:oasis:names:tc:SAML:2.0:ac:classes:MobileTwoFactorContract"
            | "http://eidas.europa.eu/LoA/low" => Some(Self::Low),
            "urn:oasis:names:tc:SAML:2.0:ac:classes:Smartcard"
            | "http://eidas.europa.eu/LoA/substantial" => Some(Self::Substantial),
            "urn:oasis:names:tc:SAML:2.0:ac:classes:SmartcardPKI"
            | "http://eidas.europa.eu/LoA/high" => Some(Self::High),
            _ => None,
        }
    }

    /// The `AuthnContextClassRef` URI used to *request* this level in an
    /// outgoing AuthnRequest. Uses the SAML `ac:classes` URN spelling from the
    /// eID SAML v4.4 §10.3 table; combined with `Comparison="minimum"` it asks
    /// the RD to authenticate at this level or higher (eID §7.6.3.2 / TVS
    /// "Checklist Testen" v2.1 T6). Round-trips through [`Self::from_uri`].
    pub fn as_request_uri(self) -> &'static str {
        match self {
            Self::Basic => "urn:oasis:names:tc:SAML:2.0:ac:classes:PasswordProtectedTransport",
            Self::Low => "urn:oasis:names:tc:SAML:2.0:ac:classes:MobileTwoFactorContract",
            Self::Substantial => "urn:oasis:names:tc:SAML:2.0:ac:classes:Smartcard",
            Self::High => "urn:oasis:names:tc:SAML:2.0:ac:classes:SmartcardPKI",
        }
    }
}

/// Minimum LoA the Kiesraad DV accepts in incoming assertions. Per eID §7.6.3.2
/// the DV MUST accept any LoA equal to or higher than the minimum registered for
/// the Service, and reject anything lower (TVS "Checklist Testen" T6). Kiesraad
/// enforces "Low" (MobileTwoFactorContract / eIDAS low) as that minimum; this
/// MUST stay in step with the level registered during TVS onboarding.
pub const MINIMUM_LOA: LevelOfAssurance = LevelOfAssurance::Low;

#[cfg(test)]
mod tests {
    use super::*;

    // -- LevelOfAssurance::from_uri (eID SAML v4.4 §10.3) --

    #[test]
    fn loa_from_uri_maps_all_spec_uris() {
        use LevelOfAssurance::{Basic, High, Low, Substantial};
        let cases = [
            (
                "urn:oasis:names:tc:SAML:2.0:ac:classes:PasswordProtectedTransport",
                Basic,
            ),
            ("http://eID.logius.nl/LoA/basic", Basic),
            (
                "urn:oasis:names:tc:SAML:2.0:ac:classes:MobileTwoFactorContract",
                Low,
            ),
            ("http://eidas.europa.eu/LoA/low", Low),
            (
                "urn:oasis:names:tc:SAML:2.0:ac:classes:Smartcard",
                Substantial,
            ),
            ("http://eidas.europa.eu/LoA/substantial", Substantial),
            ("urn:oasis:names:tc:SAML:2.0:ac:classes:SmartcardPKI", High),
            ("http://eidas.europa.eu/LoA/high", High),
        ];
        for (uri, level) in cases {
            assert_eq!(LevelOfAssurance::from_uri(uri), Some(level), "uri={uri}");
        }
        // The invented `eID.logius` substantial/high spellings are NOT valid.
        assert_eq!(
            LevelOfAssurance::from_uri("http://eID.logius.nl/LoA/substantial"),
            None
        );
        assert_eq!(LevelOfAssurance::from_uri("urn:bogus"), None);
    }

    #[test]
    fn as_request_uri_round_trips_through_from_uri() {
        use LevelOfAssurance::{Basic, High, Low, Substantial};
        for level in [Basic, Low, Substantial, High] {
            assert_eq!(
                LevelOfAssurance::from_uri(level.as_request_uri()),
                Some(level)
            );
        }
    }
}
