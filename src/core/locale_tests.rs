use super::locale::*;
use crate::Session;
use axum::{
    body::Body,
    extract::FromRequestParts,
    http::{Request, header},
};

#[tokio::test]
async fn request_locale_prefers_session() {
    let mut request = Request::builder()
        .uri("/")
        .header(header::ACCEPT_LANGUAGE, "nl-NL,nl;q=0.8")
        .body(Body::empty())
        .unwrap();
    request
        .extensions_mut()
        .insert(Session::new_test_with_locale(Locale::En));
    let (mut parts, _body) = request.into_parts();

    let locale = Locale::from_request_parts(&mut parts, &()).await.unwrap();

    assert_eq!(locale, Locale::En);
}

#[tokio::test]
async fn request_locale_falls_back_to_accept_language() {
    let request = Request::builder()
        .uri("/")
        .header(header::ACCEPT_LANGUAGE, "nl-NL,nl;q=0.8")
        .body(Body::empty())
        .unwrap();
    let (mut parts, _body) = request.into_parts();

    let locale = Locale::from_request_parts(&mut parts, &()).await.unwrap();

    assert_eq!(locale, Locale::Nl);
}
