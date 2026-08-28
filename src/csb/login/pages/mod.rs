use axum::Router;
use axum_extra::routing::RouterExt;

use crate::AppRequestState;

mod callback;
mod login;

/// Routes mounted outside the session middleware: the login start page and
/// the OAuth callback both run before a session exists. Cross-site POSTs are
/// blocked by the global fetch-metadata CSRF layer.
pub fn public_router<S: AppRequestState>() -> Router<S> {
    Router::new()
        .typed_get(login::login_start::<S>)
        .typed_post(login::login_submit::<S>)
        .typed_get(callback::callback::<S>)
}

#[cfg(test)]
mod tests {
    use axum::{
        Router,
        body::Body,
        http::{Request, StatusCode, header},
        response::Response,
    };
    use tower::ServiceExt;

    use crate::{
        AppState,
        csb::login::{pending_state_id, state_cookie::STATE_COOKIE_NAME, test_support},
        test_utils::response_body_string,
    };

    async fn test_app() -> (AppState, Router) {
        let state = AppState::new_for_tests_with_config(test_support::github_test_config()).await;
        let app = crate::app::router::create(state.clone()).with_state(state.clone());
        (state, app)
    }

    /// The state cookie's value from the response's `Set-Cookie` headers.
    fn state_cookie_value(response: &Response) -> String {
        response
            .headers()
            .get_all(header::SET_COOKIE)
            .iter()
            .filter_map(|value| value.to_str().ok())
            .filter_map(|value| value.split(';').next())
            .filter_map(|pair| pair.split_once('='))
            .find(|(name, _)| *name == STATE_COOKIE_NAME)
            .map(|(_, value)| value.to_string())
            .expect("state cookie")
    }

    #[tokio::test]
    async fn login_page_is_reachable_without_session() {
        let (_state, app) = test_app().await;

        let request = Request::builder()
            .uri("/csb/login")
            .body(Body::empty())
            .expect("valid request");
        let response = app.oneshot(request).await.expect("response");

        assert_eq!(response.status(), StatusCode::OK);
        let body = response_body_string(response).await;
        assert!(body.contains("action=\"/csb/login\""));
    }

    #[tokio::test]
    async fn login_routes_answer_not_found_when_unconfigured() {
        let state = AppState::new_for_tests().await;
        let app = crate::app::router::create(state.clone()).with_state(state);

        for uri in ["/csb/login", "/csb/login/callback"] {
            let request = Request::builder()
                .uri(uri)
                .body(Body::empty())
                .expect("valid request");
            let response = app.clone().oneshot(request).await.expect("response");

            assert_eq!(response.status(), StatusCode::NOT_FOUND, "{uri}");
        }
    }

    /// The POST mints a nonce that is registered server-side, bound to the
    /// browser via the state cookie, and carried to GitHub's consent page.
    #[tokio::test]
    async fn login_submit_redirects_to_github_with_bound_state() {
        let (state, app) = test_app().await;

        let request = Request::builder()
            .method("POST")
            .uri("/csb/login")
            .body(Body::empty())
            .expect("valid request");
        let response = app.oneshot(request).await.expect("response");

        assert_eq!(response.status(), StatusCode::SEE_OTHER);
        let location = response
            .headers()
            .get(header::LOCATION)
            .expect("redirect location")
            .to_str()
            .expect("ascii location");
        assert!(location.starts_with("https://github.com/login/oauth/authorize?"));
        assert!(location.contains("client_id=Iv1.testclient"));

        let nonce = location
            .split_once("state=")
            .and_then(|(_, rest)| rest.split('&').next())
            .expect("state in redirect");
        assert_eq!(state_cookie_value(&response), nonce);
        assert!(
            state
                .pending_requests
                .consume_if_pending(&pending_state_id(nonce))
                .await
        );
    }
}
