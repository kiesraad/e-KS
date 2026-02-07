use axum::extract::{FromRef, FromRequestParts, Path};
use serde::Deserialize;

use crate::{
    AppError, AppStore,
    authorised_agents::{AuthorisedAgent, AuthorisedAgentId},
};

#[derive(Deserialize)]
struct AuthorisedAgentPathParams {
    #[serde(alias = "agent_id")]
    agent_id: AuthorisedAgentId,
}

impl<S> FromRequestParts<S> for AuthorisedAgent
where
    S: Clone + Send + Sync + 'static,
    AppStore: FromRef<S>,
{
    type Rejection = AppError;

    async fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        state: &S,
    ) -> Result<Self, Self::Rejection> {
        let store = AppStore::from_ref(state);
        let Path(AuthorisedAgentPathParams { agent_id }) =
            Path::<AuthorisedAgentPathParams>::from_request_parts(parts, state).await?;

        store.get_authorised_agent(agent_id)
    }
}
