// SAML 2.0 (OASIS saml-core-2.0-os §1.3), SAML metadata and SOAP 1.1 namespaces.
pub const NS_SAMLP: &str = "urn:oasis:names:tc:SAML:2.0:protocol";
pub const NS_SAML: &str = "urn:oasis:names:tc:SAML:2.0:assertion";
pub const NS_MD: &str = "urn:oasis:names:tc:SAML:2.0:metadata";
pub const NS_SOAP: &str = "http://schemas.xmlsoap.org/soap/envelope/";
// XML-DSig (W3C xmldsig-core §4) and XML-Enc (W3C xmlenc-core §3) namespaces:
// element lookups in signatures / EncryptedID are matched against these.
pub const NS_DSIG: &str = "http://www.w3.org/2000/09/xmldsig#";
pub const NS_XENC: &str = "http://www.w3.org/2001/04/xmlenc#";

// eID §3.1.1: Bindings used in the message flows. The RD IdP metadata is
// queried for endpoints by these binding URIs (idp_metadata.rs).
pub const BINDING_HTTP_POST: &str = "urn:oasis:names:tc:SAML:2.0:bindings:HTTP-POST";
pub const BINDING_SOAP: &str = "urn:oasis:names:tc:SAML:2.0:bindings:SOAP";

// SAML core §3.2.2.2: the Success status code.
pub const STATUS_SUCCESS: &str = "urn:oasis:names:tc:SAML:2.0:status:Success";

// eID §7.6.3: the Subject NameID MUST contain a TransientID.
pub const NAMEID_TRANSIENT: &str = "urn:oasis:names:tc:SAML:2.0:nameid-format:transient";

// eID §7.6.3.4.4: a decrypted EncryptedID NameID MUST use the persistent format.
pub const NAMEID_PERSISTENT: &str = "urn:oasis:names:tc:SAML:2.0:nameid-format:persistent";

// eID §7.6.3.3: SubjectConfirmation Method MUST be bearer.
pub const SUBJECT_CONFIRMATION_BEARER: &str = "urn:oasis:names:tc:SAML:2.0:cm:bearer";

// eID §7.3 / §7.6.3.4: eID attribute names, matched on incoming attributes. The
// AuthnRequest/metadata templates also emit ServiceUUID and IntendedAudience
// names as literals.
pub const EID_SERVICE_UUID: &str = "urn:nl-eid-gdi:1.0:ServiceUUID";
pub const EID_ACTING_SUBJECT_ID: &str = "urn:nl-eid-gdi:1.0:ActingSubjectID";
pub const EID_LEGAL_SUBJECT_ID: &str = "urn:nl-eid-gdi:1.0:LegalSubjectID";

// eID §10.1: the three identifier types a decrypted EncryptedID NameID may
// declare in its `@NameQualifier` (§7.6.3.4.4). Anything else is not an
// identifier type this DV knows how to interpret.
pub const EID_ID_TYPES: &[&str] = &[
    "urn:nl-eid-gdi:1.0:id:legacy-BSN",
    "urn:nl-eid-gdi:1.0:id:BSN",
    "urn:nl-eid-gdi:1.0:id:Pseudonym",
];

// eID §9.5: NTP advised; allow a small skew on @NotOnOrAfter / @NotBefore checks.
pub const CLOCK_SKEW_SECONDS: i64 = 30;

// Max age of an incoming message by its @IssueInstant / @AuthnInstant. Generous:
// the artifact round-trip adds latency and replay is already blocked by
// consume-once InResponseTo + the NotOnOrAfter windows, so this only rejects
// stale messages.
pub const MESSAGE_FRESHNESS_SECONDS: i64 = 300;
