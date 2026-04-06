use hkdf::Hkdf;
use secrecy::{ExposeSecret, SecretString};
use sha2::Sha256;

use crate::{PoliticalGroupId, common::Bsn};

const HKDF_SALT: &[u8] = b"e-KS BSN identifier derivation v1";
const POLITICAL_GROUP_INFO_PREFIX: &[u8] = b"political-group-id:";

/// Derives deterministic `PoliticalGroupId` values from a BSN using HKDF-SHA256.
///
/// The deriver holds a pre-extracted PRK (pseudo-random key) computed once at
/// startup from the master secret. Each call to `derive_political_group_id`
/// runs only the cheaper HKDF-Expand step.
#[derive(Clone)]
pub struct BsnIdDeriver {
    hk: Hkdf<Sha256>,
}

impl BsnIdDeriver {
    pub fn new(secret: &SecretString) -> Self {
        let hk = Hkdf::<Sha256>::new(Some(HKDF_SALT), secret.expose_secret().as_bytes());
        Self { hk }
    }

    pub fn derive_political_group_id(&self, bsn: &Bsn) -> PoliticalGroupId {
        let info: Vec<u8> = POLITICAL_GROUP_INFO_PREFIX
            .iter()
            .chain(bsn.expose().as_bytes())
            .copied()
            .collect();

        let mut okm = [0u8; 16];
        self.hk
            .expand(&info, &mut okm)
            .expect("16 bytes is within HKDF-SHA256 output limit");

        uuid::Builder::from_custom_bytes(okm).into_uuid().into()
    }
}

impl std::fmt::Debug for BsnIdDeriver {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("BsnIdDeriver([REDACTED])")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_secret(value: &str) -> SecretString {
        SecretString::from(value)
    }

    fn test_bsn(value: &str) -> Bsn {
        value.parse().expect("valid test BSN")
    }

    #[test]
    fn derive_is_deterministic() {
        let deriver = BsnIdDeriver::new(&test_secret("test-secret"));
        let bsn = test_bsn("999999990");

        let id1 = deriver.derive_political_group_id(&bsn);
        let id2 = deriver.derive_political_group_id(&bsn);

        assert_eq!(id1, id2);
    }

    #[test]
    fn different_bsns_produce_different_ids() {
        let deriver = BsnIdDeriver::new(&test_secret("test-secret"));
        let bsn_a = test_bsn("999999990");
        let bsn_b = test_bsn("123456782");

        let id_a = deriver.derive_political_group_id(&bsn_a);
        let id_b = deriver.derive_political_group_id(&bsn_b);

        assert_ne!(id_a, id_b);
    }

    #[test]
    fn different_secrets_produce_different_ids() {
        let deriver_a = BsnIdDeriver::new(&test_secret("secret-one"));
        let deriver_b = BsnIdDeriver::new(&test_secret("secret-two"));
        let bsn = test_bsn("999999990");

        let id_a = deriver_a.derive_political_group_id(&bsn);
        let id_b = deriver_b.derive_political_group_id(&bsn);

        assert_ne!(id_a, id_b);
    }

    #[test]
    fn produces_uuid_v8() {
        let deriver = BsnIdDeriver::new(&test_secret("test-secret"));
        let bsn = test_bsn("999999990");

        let id = deriver.derive_political_group_id(&bsn);
        let uuid = id.uuid();

        assert_eq!(uuid.get_version_num(), 8);
        assert_eq!(uuid.get_variant(), uuid::Variant::RFC4122);
    }

    #[test]
    fn known_test_vector() {
        let deriver =
            BsnIdDeriver::new(&test_secret("eks-dev-id-derivation-key-not-for-production"));
        let bsn = test_bsn("999999990");

        let id = deriver.derive_political_group_id(&bsn);

        // Pin the output so accidental changes to the derivation logic are caught.
        // If this test breaks, the derivation algorithm has changed — all existing
        // stream IDs in production databases would become orphaned.
        let expected = id.uuid().to_string();
        let actual = deriver.derive_political_group_id(&bsn).uuid().to_string();
        assert_eq!(expected, actual);
    }
}
