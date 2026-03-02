use crate::{AppState, Locale, Session, common::SwitchLanguagePath};
use axum::{extract::State, response::Redirect};
use axum_extra::{TypedHeader, extract::Form, headers};
use serde::Deserialize;

#[derive(Default, Deserialize, Clone, Debug)]
pub struct LanguageSwitch {
    lang: Locale,
}

pub async fn switch_language(
    _: SwitchLanguagePath,
    TypedHeader(referer): TypedHeader<headers::Referer>,
    State(state): State<AppState>,
    mut session: Session,
    Form(form): Form<LanguageSwitch>,
) -> Redirect {
    session.locale = form.lang;
    state.sessions.insert(session);

    Redirect::to(&referer.to_string())
}

#[cfg(test)]
mod tests {
    use axum::{
        Router,
        body::Body,
        http::{Request, StatusCode, header},
        middleware,
    };
    use axum_extra::routing::RouterExt;
    use tower::ServiceExt;

    use crate::AppState;

    use super::*;

    #[tokio::test]
    async fn switch_language_updates_session_and_redirects() {
        let state = AppState::new_for_tests().await;
        let app = Router::new()
            .typed_post(switch_language)
            .layer(middleware::from_fn_with_state(
                state.clone(),
                crate::session_middleware,
            ))
            .with_state(state.clone());

        let request = Request::builder()
            .method("POST")
            .uri("/language")
            .header(header::REFERER, "https://example.com/return")
            .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
            .body(Body::from("lang=en"))
            .unwrap();

        let response = app.oneshot(request).await.expect("response");

        assert_eq!(response.status(), StatusCode::SEE_OTHER);
        assert_eq!(
            response.headers().get(header::LOCATION).unwrap(),
            "https://example.com/return"
        );
        let set_cookie = response
            .headers()
            .get(header::SET_COOKIE)
            .and_then(|value| value.to_str().ok())
            .expect("set-cookie header");
        assert!(set_cookie.contains(crate::SESSION_COOKIE_NAME));

        let token = set_cookie
            .split(';')
            .next()
            .and_then(|pair| pair.split_once('='))
            .map(|(_, value)| value)
            .expect("session token");
        let session = state.sessions.get(token).expect("session");
        assert_eq!(session.locale, Locale::En);
    }
}
