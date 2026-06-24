//! Placeholder request handlers.
//!
//! The SSO/SLO *start* entry points ([`handle_login`], [`handle_logout`]) keep
//! the exact signatures the embedding `eks` crate mounts, so the application
//! compiles unchanged. Instead of a real SAML round-trip, [`handle_login`] logs
//! the user straight in with a random test BSN (just like the `eks` dev login).
//! The SAML protocol endpoints mounted by [`crate::router`] respond with
//! `501 Not Implemented` until the real SP implementation lands.

use axum::{
    extract::{FromRef, State},
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Redirect, Response},
};
use axum_extra::extract::CookieJar;
use rand::RngExt;
use secrecy::SecretString;

use crate::state::{AuthServiceState, AuthState, SubjectId};

const PLACEHOLDER_MSG: &str = "auth-service placeholder: SAML SP not available in this build";

/// `NameQualifier` recorded on the minted subject. The real SP fills this from
/// the validated assertion; here it is a fixed placeholder.
const PLACEHOLDER_NAME_QUALIFIER: &str = "placeholder";

/// Start "SAML SSO". The full implementation builds a signed AuthnRequest and
/// auto-POSTs it to the RD; this placeholder skips the round-trip and logs the
/// user straight in with a random valid test BSN (mirroring the `eks` dev
/// login), delegating to [`AuthState::on_authenticated`]. Generic over the
/// embedding state `S` so the call site `handle_login::<AppState>` keeps
/// compiling.
pub async fn handle_login<S>(
    State(state): State<S>,
    State(_auth_state): State<AuthServiceState>,
    jar: CookieJar,
    headers: HeaderMap,
) -> Response
where
    S: AuthState,
    AuthServiceState: FromRef<S>,
{
    let subject_id = SubjectId {
        value: random_bsn(),
        name_qualifier: PLACEHOLDER_NAME_QUALIFIER.to_string(),
    };
    // A NameID is needed at logout to clear the session cookie cleanly; the real
    // SP carries the IdP's TransientID, the placeholder mints a random one.
    let name_id = Some(random_name_id());
    state
        .on_authenticated(subject_id, name_id, jar, &headers)
        .await
}

/// SP-initiated logout (eID §7.7.1). The full implementation builds a signed
/// LogoutRequest for the browser to POST to the RD; this placeholder still tears
/// down the local session (so sign-out keeps working) and redirects home.
pub async fn handle_logout<S>(
    State(state): State<S>,
    State(_auth_state): State<AuthServiceState>,
    jar: CookieJar,
) -> Response
where
    S: AuthState,
    AuthServiceState: FromRef<S>,
{
    let Some((_name_id, cleared_jar)) = state.logout_session(jar).await else {
        return Redirect::to("/").into_response();
    };
    (cleared_jar, Redirect::to("/")).into_response()
}

/// `GET /saml/sp/metadata` serves the signed SP metadata in the full
/// implementation; placeholder.
pub async fn handle_metadata() -> Response {
    (StatusCode::NOT_IMPLEMENTED, PLACEHOLDER_MSG).into_response()
}

/// `GET /saml/sp/acs` Assertion Consumer Service in the full implementation;
/// placeholder.
pub async fn handle_acs() -> Response {
    (StatusCode::NOT_IMPLEMENTED, PLACEHOLDER_MSG).into_response()
}

/// `POST /saml/sp/logout` receives the LogoutResponse in the full
/// implementation; placeholder.
pub async fn handle_sls() -> Response {
    (StatusCode::NOT_IMPLEMENTED, PLACEHOLDER_MSG).into_response()
}

/// Generate a random but valid BSN (Burgerservicenummer) as a `SecretString`,
/// mirroring the `eks` dev login. A valid BSN is 9 digits satisfying the
/// 11-proof check (weights 9,8,7,6,5,4,3,2,-1 applied left-to-right, weighted
/// sum divisible by 11); a `999` prefix keeps it in the designated test range.
fn random_bsn() -> SecretString {
    let mut rng = rand::rng();
    loop {
        let prefix: Vec<u32> = vec![9, 9, 9];
        let random_part: Vec<u32> = (0..5).map(|_| rng.random_range(0..10)).collect();
        let digits: Vec<u32> = prefix.into_iter().chain(random_part).collect();
        let weights = [9, 8, 7, 6, 5, 4, 3, 2];
        let partial_sum: i32 = digits
            .iter()
            .zip(weights.iter())
            .map(|(&d, &w)| d as i32 * w)
            .sum();

        // last digit weight is -1, so: (partial_sum - last_digit) % 11 == 0
        let remainder = partial_sum.rem_euclid(11);
        if remainder > 9 {
            continue;
        }
        let last_digit = remainder as u32;

        let bsn: String = digits
            .iter()
            .chain(std::iter::once(&last_digit))
            .map(|d| char::from_digit(*d, 10).unwrap())
            .collect();

        return SecretString::from(bsn);
    }
}

/// Mint a random opaque NameID for the placeholder session.
fn random_name_id() -> String {
    let mut rng = rand::rng();
    let value: u128 = rng.random();
    format!("placeholder-{value:032x}")
}
