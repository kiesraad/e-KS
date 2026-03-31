use askama::Template;
use axum::{
    http::{HeaderMap, HeaderValue, header::CONTENT_SECURITY_POLICY},
    response::{Html, IntoResponse},
};
use base64::{Engine, prelude::BASE64_STANDARD};
use url::Url;

use crate::AppError;

#[derive(Template)]
#[template(path = "login.html", print = "code")]
pub struct LoginAutoPost<'a> {
    pub login_url: &'a str,
    pub saml_request: &'a str,
    pub relay_state: Option<&'a str>,
}

pub fn login_auto_post(
    login_url: &str,
    authn_request_xml: &[u8],
) -> Result<impl IntoResponse + use<>, AppError> {
    // allow form actions with SAML server
    let login_origin = Url::parse(login_url)
        .map_err(|_| AppError::InternalServerError)?
        .origin()
        .ascii_serialization();
    let mut headers = HeaderMap::new();
    headers.insert(
        CONTENT_SECURITY_POLICY,
        HeaderValue::from_str(&format!(
            "default-src 'none'; base-uri 'none'; connect-src 'self'; form-action 'self' {login_origin}; script-src 'self'; style-src 'self'; font-src 'self'; img-src 'self'; frame-ancestors 'none';"
        )).map_err(|_| AppError::InternalServerError)?,
    );

    // render page
    let html = LoginAutoPost {
        login_url,
        saml_request: &BASE64_STANDARD.encode(authn_request_xml),
        relay_state: None,
    }
    .render()?;
    Ok((headers, Html(html)))
}
