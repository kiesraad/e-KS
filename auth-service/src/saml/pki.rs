//! PKIoverheid trust material.

/// The single root CA pinned for the mTLS SOAP back-channel, used to verify the
/// ARS server certificate (eID §9.4); see [`crate::bindings::soap`]. PEM-encoded.
///
/// Which root is pinned is fixed at compile time by the `tvs-mock` feature:
///   - `tvs-mock`: the repo's self-signed test CA (`auth-service/fixtures/ca.pem`),
///     which signs the standalone TVS mock's back-channel server certificate.
///   - otherwise: the Staat der Nederlanden Private Root CA - G1, to which real
///     PKIoverheid server certificates chain. This root is not in the Mozilla
///     store, so relying on system roots fails with UnknownIssuer.
#[cfg(feature = "tvs-mock")]
pub const BACKCHANNEL_ROOT_CA_PEM: &[u8] = include_bytes!("../../fixtures/ca.pem");

#[cfg(not(feature = "tvs-mock"))]
pub const BACKCHANNEL_ROOT_CA_PEM: &[u8] = include_bytes!("../../pkioverheid_private_root_g1.pem");

/// Trust anchor(s) the RD **metadata signing certificate** must chain to
/// (eID §9.1/§9.2). The signing cert in the RD metadata is validated against
/// these roots (plus [`RD_METADATA_INTERMEDIATES`]) rather than being trusted on
/// its own self-signature, so a spoofed metadata document signed with an attacker
/// key is rejected even if the HTTPS fetch is subverted. PEM-encoded.
///
/// The same root as [`BACKCHANNEL_ROOT_CA_PEM`], so the `tvs-mock` feature picks
/// the test CA (which directly issues the mock RD's signing certificate, no
/// intermediates) and production picks the Staat der Nederlanden Private Root
/// CA - G1 the TVS RD signing cert chains to.
pub const RD_METADATA_TRUST_ROOTS: &[&[u8]] = &[BACKCHANNEL_ROOT_CA_PEM];

/// Intermediate CA certificate(s) bridging the RD metadata signing certificate to
/// a [`RD_METADATA_TRUST_ROOTS`] anchor. The RD metadata ships only its leaf
/// signing certificate, so the path-building intermediates must be supplied here.
///
///   - `tvs-mock`: none (the test CA issues the mock signing cert directly).
///   - otherwise: `Staat der Nederlanden Private Services CA - G1` and
///     `DigiCert QuoVadis PKIoverheid Private Services CA - 2023`, the two
///     intermediates between the TVS RD signing cert and the Private Root G1
///     (both valid through 2028; refresh when the RD's issuing CA rotates).
#[cfg(feature = "tvs-mock")]
pub const RD_METADATA_INTERMEDIATES: &[&[u8]] = &[];

#[cfg(not(feature = "tvs-mock"))]
pub const RD_METADATA_INTERMEDIATES: &[&[u8]] = &[
    include_bytes!("../../staat_private_services_ca_g1.pem"),
    include_bytes!("../../pkioverheid_private_services_ca_2023.pem"),
];
