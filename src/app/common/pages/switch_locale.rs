use crate::{
    Locale,
    common::{LOCALE_COOKIE_NAME, SwitchLanguagePath},
};
use axum::response::Redirect;
use axum_extra::{
    TypedHeader,
    extract::{CookieJar, Form, cookie::Cookie},
    headers,
};
use serde::Deserialize;

#[derive(Default, Deserialize, Clone, Debug)]
pub struct LanguageSwitch {
    lang: Locale,
}

pub async fn switch_language(
    _: SwitchLanguagePath,
    TypedHeader(referer): TypedHeader<headers::Referer>,
    mut cookie_jar: CookieJar,
    Form(form): Form<LanguageSwitch>,
) -> (CookieJar, Redirect) {
    cookie_jar = cookie_jar.add(Cookie::new(LOCALE_COOKIE_NAME, form.lang.as_str()));

    (cookie_jar, Redirect::to(&referer.to_string()))
}

#[cfg(test)]
mod tests {
    use axum::{
        Router,
        body::Body,
        http::{Request, StatusCode, header},
    };
    use axum_extra::routing::RouterExt;
    use tower::ServiceExt;

    use crate::AppState;

    use super::*;

    #[tokio::test]
    async fn switch_language_sets_cookie_and_redirects() {
        let app = Router::new()
            .typed_post(switch_language)
            .with_state(AppState::new_for_tests().await);

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
            .unwrap_or_default();
        assert!(set_cookie.contains("LANGUAGE=en"));
    }
}
