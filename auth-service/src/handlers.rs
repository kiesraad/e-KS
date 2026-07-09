//! Placeholder request handlers. Login/logout keep the real signatures but skip
//! the SAML round-trip; protocol endpoints return `501 Not Implemented`.

use axum::{
    extract::{FromRef, State},
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Redirect, Response},
};
use axum_extra::extract::CookieJar;
use rand::RngExt;
use secrecy::SecretString;

use crate::{
    SamlAcsPath, SamlLogoutPath, SamlMetadataPath,
    state::{AuthServiceState, AuthState, SubjectId},
};

const PLACEHOLDER_MSG: &str = "auth-service placeholder: SAML SP not available in this build";

const PLACEHOLDER_NAME_QUALIFIER: &str = "placeholder";

/// Placeholder SSO entry point. Skips the SAML round-trip and logs the user in
/// with a random test BSN, mirroring the `eks` dev login.
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
    let name_id = random_name_id();
    state
        .on_authenticated(subject_id, name_id, jar, &headers)
        .await
}

/// Placeholder SLO entry point (eID §7.7.1). Tears down the local session and
/// redirects to `post_logout_redirect` without sending a LogoutRequest to the IdP.
pub async fn handle_logout<S>(state: &S, jar: CookieJar, post_logout_redirect: &str) -> Response
where
    S: AuthState,
{
    let (cleared_jar, _name_id) = state.logout_session(jar).await;
    (cleared_jar, Redirect::to(post_logout_redirect)).into_response()
}

/// `GET /saml/sp/metadata` signed SP metadata placeholder
pub async fn handle_metadata(_: SamlMetadataPath) -> Response {
    (StatusCode::NOT_IMPLEMENTED, PLACEHOLDER_MSG).into_response()
}

/// `GET /saml/sp/acs` Assertion Consumer Service placeholder
pub async fn handle_acs(_: SamlAcsPath) -> Response {
    (StatusCode::NOT_IMPLEMENTED, PLACEHOLDER_MSG).into_response()
}

/// `POST /saml/sp/logout` placeholder
pub async fn handle_sls(_: SamlLogoutPath) -> Response {
    (StatusCode::NOT_IMPLEMENTED, PLACEHOLDER_MSG).into_response()
}

/// Random valid BSN: 9 digits with a 999-prefix (test range), satisfying the 11-proof check
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
