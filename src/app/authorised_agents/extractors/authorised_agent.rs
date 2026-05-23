use axum::extract::Path;
use serde::Deserialize;

use crate::{
    app::request_extractor,
    authorised_agents::{AuthorisedAgent, AuthorisedAgentId},
};

#[derive(Deserialize)]
struct AuthorisedAgentPathParams {
    #[serde(alias = "agent_id")]
    agent_id: AuthorisedAgentId,
}

request_extractor!(AuthorisedAgent, |store, parts, state| {
    let Path(AuthorisedAgentPathParams { agent_id }) =
        Path::<AuthorisedAgentPathParams>::from_request_parts(parts, state).await?;

    store.get_authorised_agent(agent_id)
});

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        Router,
        body::Body,
        http::{Request, StatusCode},
        routing::get,
    };
    use tower::ServiceExt;

    use crate::{
        AppState, AppStore,
        test_utils::{response_body_string, sample_authorised_agent},
    };

    #[tokio::test]
    async fn authorised_agent_extractor_loads_agent() {
        let authorised_agent = sample_authorised_agent(AuthorisedAgentId::new());

        let app_state = AppState::new_for_tests().await;
        let store = AppStore::new_for_test();
        authorised_agent
            .create(&store)
            .await
            .expect("create authorised agent");

        let app = Router::new()
            .route(
                "/political-group/authorised-agents/{agent_id}",
                get(|authorised_agent: AuthorisedAgent| async move {
                    authorised_agent.name.last_name.to_string()
                }),
            )
            .with_state(app_state);

        let mut request = Request::builder()
            .uri(format!(
                "/political-group/authorised-agents/{}",
                authorised_agent.id
            ))
            .body(Body::empty())
            .unwrap();
        request.extensions_mut().insert(store.clone());

        let response = app.oneshot(request).await.expect("response");

        assert_eq!(response.status(), StatusCode::OK);
        let body = response_body_string(response).await;
        assert!(body.contains("Jansen"));
    }
}
