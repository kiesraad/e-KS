use askama::Template;
use axum::{
    extract::State,
    response::{IntoResponse, Redirect, Response},
};

use crate::{
    AnyLocale, AppError, AppState, Context, CsrfToken, ElectionConfig, HtmlTemplate, Province,
    Session, TokenValue, WaterCouncil, common::SwitchElectionForm, filters,
};

use super::{IndexPath, SwitchElectionPath};

#[derive(Template)]
#[template(path = "common/pages/switch_election.html")]
struct SwitchElectionTemplate {
    elections: Vec<ElectionConfig>,
    existing_elections: Vec<ElectionConfig>,
    current_election: ElectionConfig,
    title_locale: AnyLocale,
    current_type: &'static str,
    selected_region: Option<&'static str>,
    provinces: &'static [Province],
    water_councils: &'static [WaterCouncil],
    csrf_token: CsrfToken,
}

pub async fn switch_election(
    _: SwitchElectionPath,
    State(state): State<AppState>,
    context: Context,
) -> Result<Response, AppError> {
    let existing_elections = state
        .existing_elections_for_code(&context.session.id_code)
        .await?;

    let csrf_token = context.session.csrf_tokens.issue();
    let elections = ElectionConfig::type_options();

    Ok(HtmlTemplate(
        SwitchElectionTemplate {
            current_election: context.election,
            title_locale: AnyLocale::from(context.session.locale),
            current_type: context.election.code(),
            selected_region: context.election.region_code(),
            provinces: Province::ALL,
            water_councils: WaterCouncil::ALL,
            elections,
            existing_elections,
            csrf_token,
        },
        context,
    )
    .into_response())
}

pub async fn switch_election_submit(
    _: SwitchElectionPath,
    State(state): State<AppState>,
    mut session: Session,
    axum::Form(form): axum::Form<SwitchElectionForm>,
) -> Result<Response, AppError> {
    if !session
        .csrf_tokens
        .consume(&TokenValue(form.csrf_token.clone()))
    {
        return Err(AppError::CsrfTokenInvalid);
    }

    let Some(election) = form.into_election_config() else {
        return Ok(Redirect::to(&SwitchElectionPath.to_string()).into_response());
    };

    let stream_id = state
        .id_deriver
        .derive_stream_id(&session.id_code, election);

    // Short-circuit if already on this election (same ID code + election = same stream)
    if Some(stream_id) == session.stream_id {
        return Ok(Redirect::to(&IndexPath.to_string()).into_response());
    }

    // Ensure the stream/store exists for the new election
    state.store_for_stream(stream_id, election, false).await?;

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

    use crate::{AppState, ElectionConfig, Province, session_middleware, store_middleware};
    use secrecy::SecretString;

    use super::*;

    #[tokio::test]
    async fn switch_election_submit_changes_session_election() {
        let state = AppState::new_for_tests().await;
        let app = Router::new()
            .typed_get(switch_election)
            .typed_post(switch_election_submit)
            .layer(middleware::from_fn_with_state(
                state.clone(),
                store_middleware,
            ))
            .layer(middleware::from_fn_with_state(
                state.clone(),
                session_middleware,
            ))
            .with_state(state.clone());

        // Pre-create a session with a stream
        let id_code: SecretString = "999999990".into();
        let stream_id = state
            .id_deriver
            .derive_stream_id(&id_code, ElectionConfig::EK27);
        state
            .store_for_stream(stream_id, ElectionConfig::EK27, false)
            .await
            .expect("store");

        let mut session = crate::Session::new(&id_code);
        session.set_stream_id(stream_id);
        let token_value = session.token().to_exposed_string();
        let csrf_token = session.csrf_tokens.issue();
        state.sessions.insert(session);

        let cookie = format!("{}={}", crate::SESSION_COOKIE_NAME, token_value);

        // Submit switch to PS27 Groningen
        let body = format!(
            "csrf_token={}&election=PS27&region_province=GR",
            csrf_token.value
        );
        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/switch-election")
                    .header(header::COOKIE, &cookie)
                    .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
                    .body(Body::from(body))
                    .unwrap(),
            )
            .await
            .expect("response");

        assert_eq!(response.status(), StatusCode::SEE_OTHER);
        assert_eq!(response.headers().get(header::LOCATION).unwrap(), "/");

        // Verify session stream was updated
        let session = state.sessions.get(&token_value).expect("session");
        let expected_stream_id = state
            .id_deriver
            .derive_stream_id(&id_code, ElectionConfig::PS27(Province::GR));
        assert_eq!(session.stream_id, Some(expected_stream_id));
    }
}
