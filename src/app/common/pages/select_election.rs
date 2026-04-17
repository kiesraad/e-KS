use askama::Template;
use axum::{
    extract::State,
    http::{HeaderValue, header},
    response::{Html, IntoResponse, Redirect, Response},
};

use crate::{
    AnyLocale, AppError, AppState, Context, Locale, Province, Session, TokenValue, WaterCouncil,
    common::SelectElectionForm, filters,
};

use super::{IndexPath, SelectElectionPath};

#[derive(Template)]
#[template(path = "common/pages/select_election.html")]
struct SelectElectionTemplate {
    elections: Vec<crate::ElectionConfig>,
    title_locale: AnyLocale,
    provinces: &'static [Province],
    water_councils: &'static [WaterCouncil],
    csrf_token: crate::CsrfToken,
    fixtures: bool,
}

struct LocaleValues {
    locale: Locale,
}

impl askama::Values for LocaleValues {
    fn get_value<'a>(&'a self, key: &str) -> Option<&'a dyn std::any::Any> {
        match key {
            "locale" => Some(&self.locale as &dyn std::any::Any),
            _ => None,
        }
    }
}

pub async fn select_election(
    _: SelectElectionPath,
    session: Session,
    State(state): State<AppState>,
) -> Result<Response, AppError> {
    if session.stream_id.is_some() {
        return Ok(Redirect::to(&IndexPath.to_string()).into_response());
    }

    let csrf_token = session.csrf_tokens.issue();
    state.sessions.insert(session.clone());

    let template = SelectElectionTemplate {
        elections: crate::ElectionConfig::type_options(),
        title_locale: AnyLocale::from(session.locale),
        provinces: Province::ALL,
        water_councils: WaterCouncil::ALL,
        csrf_token,
        fixtures: cfg!(feature = "fixtures"),
    };

    let values = LocaleValues {
        locale: session.locale,
    };
    let html = template
        .render_with_values(&values)
        .map_err(AppError::TemplateError)?;

    let mut response = Html(html).into_response();
    response
        .headers_mut()
        .insert(header::CACHE_CONTROL, HeaderValue::from_static("no-store"));
    Ok(response)
}

pub async fn select_election_submit(
    _: SelectElectionPath,
    State(state): State<AppState>,
    mut session: Session,
    axum::Form(form): axum::Form<SelectElectionForm>,
) -> Result<Response, AppError> {
    if !session
        .csrf_tokens
        .consume(&TokenValue(form.csrf_token.clone()))
    {
        return Err(AppError::CsrfTokenInvalid);
    }

    let Some(election) = form.into_election_config() else {
        return Ok(Redirect::to(&SelectElectionPath.to_string()).into_response());
    };

    let stream_id = state
        .id_deriver
        .derive_stream_id(&session.id_code, election);

    let _store = state
        .store_for_stream(stream_id, election, form.load_fixtures())
        .await?;

    session.set_stream_id(stream_id);
    state.sessions.insert(session);

    Ok(Redirect::to(&IndexPath.to_string()).into_response())
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

    use crate::{AppState, ElectionConfig, Session, session_middleware};

    use super::*;

    #[tokio::test]
    async fn select_election_redirects_when_stream_already_set() {
        let state = AppState::new_for_tests().await;
        let app = Router::new()
            .typed_get(select_election)
            .layer(middleware::from_fn_with_state(
                state.clone(),
                session_middleware,
            ))
            .with_state(state.clone());

        let mut session = Session::new_test();
        session.set_stream_id(crate::StreamId::new());
        let token = session.token().to_exposed_string();
        state.sessions.insert(session);

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/select-election")
                    .header(
                        header::COOKIE,
                        format!("{}={}", crate::SESSION_COOKIE_NAME, token),
                    )
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .expect("response");

        assert_eq!(response.status(), StatusCode::SEE_OTHER);
        assert_eq!(response.headers().get(header::LOCATION).unwrap(), "/");
    }

    #[tokio::test]
    async fn select_election_submit_sets_stream_id() {
        let state = AppState::new_for_tests().await;
        let app = Router::new()
            .typed_get(select_election)
            .typed_post(select_election_submit)
            .layer(middleware::from_fn_with_state(
                state.clone(),
                session_middleware,
            ))
            .with_state(state.clone());

        let mut session = Session::new_test();
        session.id_code = secrecy::SecretString::from("999999990");
        let token = session.token().to_exposed_string();
        state.sessions.insert(session);

        // Fetch page to receive a CSRF token
        let get = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/select-election")
                    .header(
                        header::COOKIE,
                        format!("{}={}", crate::SESSION_COOKIE_NAME, token),
                    )
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .expect("response");
        assert_eq!(get.status(), StatusCode::OK);

        let session = state.sessions.get(&token).expect("session");
        let csrf = session.csrf_tokens.issue();

        let body = format!("csrf_token={}&election=EK27", csrf.value);
        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/select-election")
                    .header(
                        header::COOKIE,
                        format!("{}={}", crate::SESSION_COOKIE_NAME, token),
                    )
                    .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
                    .body(Body::from(body))
                    .unwrap(),
            )
            .await
            .expect("response");

        assert_eq!(response.status(), StatusCode::SEE_OTHER);
        assert_eq!(response.headers().get(header::LOCATION).unwrap(), "/");

        let session = state.sessions.get(&token).expect("session");
        let expected = state
            .id_deriver
            .derive_stream_id(&session.id_code.clone(), ElectionConfig::EK27);
        assert_eq!(session.stream_id, Some(expected));
    }
}
