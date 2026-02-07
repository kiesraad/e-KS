use axum::Router;
use axum_extra::routing::{RouterExt, TypedPath};
use serde::Deserialize;

use crate::{
    AppError, AppState,
    authorised_agents::{AuthorisedAgent, AuthorisedAgentId},
};

mod authorised_agent_create;
mod authorised_agent_delete;
mod authorised_agent_update;
mod authorised_agents;

#[derive(TypedPath, Deserialize)]
#[typed_path("/political-group/authorised-agents", rejection(AppError))]
pub struct AuthorisedAgentsPath;

#[derive(TypedPath)]
#[typed_path("/political-group/authorised-agents/create", rejection(AppError))]
pub struct AuthorisedAgentCreatePath;

#[derive(TypedPath, Deserialize)]
#[typed_path(
    "/political-group/authorised-agents/{agent_id}/update",
    rejection(AppError)
)]
pub struct AuthorisedAgentUpdatePath {
    pub agent_id: AuthorisedAgentId,
}

#[derive(TypedPath, Deserialize)]
#[typed_path(
    "/political-group/authorised-agents/{agent_id}/delete",
    rejection(AppError)
)]
pub struct AuthorisedAgentDeletePath {
    pub agent_id: AuthorisedAgentId,
}

impl AuthorisedAgent {
    pub fn list_path() -> String {
        AuthorisedAgentsPath {}.to_string()
    }

    pub fn create_path() -> String {
        AuthorisedAgentCreatePath {}.to_string()
    }

    pub fn update_path(&self) -> String {
        AuthorisedAgentUpdatePath { agent_id: self.id }.to_string()
    }

    pub fn delete_path(&self) -> String {
        AuthorisedAgentDeletePath { agent_id: self.id }.to_string()
    }
}

pub fn router() -> Router<AppState> {
    Router::new()
        .typed_get(authorised_agents::list_authorised_agents)
        .typed_get(authorised_agent_create::create_authorised_agent)
        .typed_post(authorised_agent_create::create_authorised_agent_submit)
        .typed_get(authorised_agent_update::update_authorised_agent)
        .typed_post(authorised_agent_update::update_authorised_agent_submit)
        .typed_post(authorised_agent_delete::delete_authorised_agent)
}
