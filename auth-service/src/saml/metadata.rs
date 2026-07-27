//! DV SAML metadata builder.

use secrecy::ExposeSecret;

use crate::{
    error::{AuthError, Result},
    keys::KeyPair,
    saml::{
        crypto::sign,
        xml_builder::{DvMetadataArgs, build_dv_metadata, generate_id},
    },
};

/// Inputs for [`build_signed_dv_metadata`]: the SP identity and endpoints plus
/// the key material to publish. The document is signed with the first signing
/// key (whose certificate is also carried in the Signature `KeyInfo`).
pub struct SignedDvMetadataArgs<'a> {
    pub entity_id: &'a str,
    pub acs_url: &'a str,
    pub slo_url: &'a str,
    pub service_name: &'a str,
    pub service_uuid: &'a str,
    pub signing_keys: &'a [KeyPair],
    /// The DV's mTLS client certificate, published alongside the SAML signing
    /// cert(s) as a `use="signing"` KeyDescriptor (eID §8.3); `None` to omit it.
    pub tls_signing_cert: Option<&'a KeyPair>,
    pub encryption_keys: &'a [KeyPair],
}

pub fn build_signed_dv_metadata(args: SignedDvMetadataArgs) -> Result<String> {
    let SignedDvMetadataArgs {
        entity_id,
        acs_url,
        slo_url,
        service_name,
        service_uuid,
        signing_keys,
        tls_signing_cert,
        encryption_keys,
    } = args;

    let signing_key = signing_keys
        .first()
        .ok_or_else(|| AuthError::Config("no DV signing key to sign metadata".to_string()))?;

    let metadata_id = generate_id();

    // Drop the TLS cert when it is already one of the signing certs (a combined
    // signing+TLS certificate) so the metadata never carries a duplicate
    // KeyDescriptor.
    let tls_extra =
        tls_signing_cert.filter(|tls| signing_keys.iter().all(|k| k.key_name != tls.key_name));

    let sk: Vec<(&str, &str)> = signing_keys
        .iter()
        .chain(tls_extra)
        .map(|k| (k.key_name.as_str(), k.cert_base64.as_str()))
        .collect();
    let ek: Vec<(&str, &str)> = encryption_keys
        .iter()
        .map(|k| (k.key_name.as_str(), k.cert_base64.as_str()))
        .collect();

    let xml = build_dv_metadata(DvMetadataArgs {
        entity_id,
        acs_url,
        slo_url,
        service_name,
        service_uuid,
        metadata_id: &metadata_id,
        signing_cert_base64: &signing_key.cert_base64,
        signing_keys: &sk,
        encryption_keys: &ek,
    })?;

    sign(&xml, signing_key.key_pem.expose_secret())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    /// Load a fixture key pair from the committed TVS test bundle.
    fn load_pair(name: &str) -> KeyPair {
        let dir = PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/fixtures"));
        let cert_pem = std::fs::read_to_string(dir.join(format!("{name}.pem"))).unwrap();
        let key_pem = std::fs::read_to_string(dir.join(format!("{name}-key.pem"))).unwrap();
        KeyPair::from_pem(cert_pem, key_pem.into())
    }

    fn build(signing: &[KeyPair], tls: Option<&KeyPair>, encryption: &[KeyPair]) -> String {
        build_signed_dv_metadata(SignedDvMetadataArgs {
            entity_id: "urn:test:dv",
            acs_url: "https://dv.test/saml/sp/acs",
            slo_url: "https://dv.test/saml/sp/logout",
            service_name: "Test DV",
            service_uuid: "f847dc11-ac24-47b2-84a8-a057440ce56d",
            signing_keys: signing,
            tls_signing_cert: tls,
            encryption_keys: encryption,
        })
        .expect("metadata must build and sign")
    }

    #[test]
    fn tls_cert_published_as_extra_signing_key_descriptor() {
        let signing = load_pair("dv-signing-1");
        let encryption = load_pair("dv-encryption-1");
        let tls = load_pair("dv-tls");

        let xml = build(
            std::slice::from_ref(&signing),
            Some(&tls),
            std::slice::from_ref(&encryption),
        );

        assert_eq!(
            xml.matches(r#"use="signing""#).count(),
            2,
            "SAML signing cert + TLS cert each get a signing KeyDescriptor"
        );
        assert_eq!(xml.matches(r#"use="encryption""#).count(), 1);
        assert!(
            xml.contains(&tls.key_name),
            "TLS cert KeyName must appear in the metadata"
        );
        assert!(xml.contains(&signing.key_name));
    }

    #[test]
    fn tls_cert_equal_to_signing_cert_is_not_duplicated() {
        let signing = load_pair("dv-signing-1");
        let encryption = load_pair("dv-encryption-1");

        // A combined signing+TLS certificate: the cert handed in as the TLS
        // cert is the same one already listed as the SAML signing key.
        let xml = build(
            std::slice::from_ref(&signing),
            Some(&signing),
            std::slice::from_ref(&encryption),
        );

        assert_eq!(
            xml.matches(r#"use="signing""#).count(),
            1,
            "a combined signing+TLS cert must not yield a duplicate KeyDescriptor"
        );
    }

    #[test]
    fn signed_dv_metadata_verifies() {
        use crate::saml::verification::verify_xml_signature;

        let signing = load_pair("dv-signing-1");
        let encryption = load_pair("dv-encryption-1");
        let tls = load_pair("dv-tls");

        let xml = build(
            std::slice::from_ref(&signing),
            Some(&tls),
            std::slice::from_ref(&encryption),
        );

        let result = verify_xml_signature(&xml, std::slice::from_ref(&signing));
        assert!(
            result.is_valid(),
            "signed DV metadata must verify with the signing key: {:?}",
            result.errors
        );
    }

    #[test]
    fn no_tls_cert_publishes_only_saml_signing_key() {
        let signing = load_pair("dv-signing-1");
        let encryption = load_pair("dv-encryption-1");

        let xml = build(
            std::slice::from_ref(&signing),
            None,
            std::slice::from_ref(&encryption),
        );

        assert_eq!(xml.matches(r#"use="signing""#).count(), 1);
    }
}
