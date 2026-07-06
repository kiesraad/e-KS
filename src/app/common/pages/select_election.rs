use askama::Template;
use axum::{
    extract::State,
    http::{HeaderValue, header},
    response::{Html, IntoResponse, Redirect, Response},
};

use crate::{
    AnyLocale, AppError, AppState, Context, Locale, Province, Scope, Session, WaterCouncil,
    common::SelectElectionForm, csb::examination::CsbExaminationOverviewPath, filters,
};

use super::{IndexPath, SelectElectionPath};

#[derive(Template)]
#[template(path = "common/pages/select_election.html")]
struct SelectElectionTemplate {
    elections: Vec<crate::ElectionConfig>,
    title_locale: AnyLocale,
    provinces: &'static [Province],
    water_councils: &'static [WaterCouncil],
    csrf_token: crate::TokenValue,
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
    if session.current_election.is_some() {
        return Ok(Redirect::to(&IndexPath.to_string()).into_response());
    }

    let csrf_token = session.csrf_token.clone();
    state.sessions.insert(session.clone()).await;

    let template = SelectElectionTemplate {
        elections: crate::ElectionConfig::type_options(),
        title_locale: AnyLocale::from(session.locale),
        provinces: Province::ALL,
        water_councils: WaterCouncil::ALL,
        csrf_token,
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
    session.consume_csrf(&form.csrf_token)?;

    // Committee sessions use CSB stores, not app stores; never create an
    // `AppStore` in their `(stream_id, election)` partition.
    if session.scope == Scope::CentralElectoralCommittee {
        return Ok(Redirect::to(&CsbExaminationOverviewPath {}.to_string()).into_response());
    }

    let Some(election) = form.into_election_config() else {
        return Ok(Redirect::to(&SelectElectionPath.to_string()).into_response());
    };

    // Only available with the `fixtures` feature: this is a test/dev shortcut
    // into the committee (CSB) scope
    #[cfg(feature = "fixtures")]
    if form.login_as_csb() {
        session.stream_id = Some(crate::StreamId::new());
        session.scope = Scope::CentralElectoralCommittee;
        session.set_current_election(election);

        if form.load_fixtures() {
            crate::csb::import::fixture::import_csb_fixture(&state, election).await?;
        }

        state.sessions.insert(session).await;

        return Ok(Redirect::to(&CsbExaminationOverviewPath {}.to_string()).into_response());
    }

    // use the stream ID derived from the authenticated login
    let Some(stream_id) = session.stream_id else {
        return Ok(Redirect::to(&SelectElectionPath.to_string()).into_response());
    };

    let _store = state
        .store_for_stream(stream_id, election, form.load_fixtures())
        .await?;

    session.set_current_election(election);
    state.sessions.insert(session).await;

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
    async fn select_election_redirects_when_election_already_set() {
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
        session.set_current_election(ElectionConfig::EK27);
        let token = session.token_string();
        state.sessions.insert(session).await;

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
    async fn select_election_submit_sets_current_election() {
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
        session.set_stream_id(crate::StreamId::new());
        let token = session.token_string();
        state.sessions.insert(session).await;

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

        let session = state
            .sessions
            .get(&token)
            .await
            .expect("load session")
            .expect("session");
        let csrf = session.csrf_token.clone();

        let body = format!("csrf_token={csrf}&election=EK27");
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

        let session = state
            .sessions
            .get(&token)
            .await
            .expect("load session")
            .expect("session");
        assert_eq!(session.current_election, Some(ElectionConfig::EK27));
    }
}
