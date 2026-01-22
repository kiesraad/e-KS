use crate::{
    TokenValue,
    form::*,
    political_group::structs::{AuthorisedAgent, AuthorisedAgentId},
};
use chrono::Utc;
use serde::Deserialize;
use validate::Validate;

#[derive(Default, Deserialize, Debug, Validate)]
#[validate(
    target = "AuthorisedAgent",
    build = "AuthorisedAgentForm::build_authorised_agent"
)]
#[serde(default)]
pub struct AuthorisedAgentForm {
    #[validate(with = "validate_length(2, 255)", with = "validate_teletex_chars()")]
    last_name: String,
    #[validate(
        with = "validate_length(1, 255)",
        with = "validate_teletex_chars()",
        optional
    )]
    last_name_prefix: String,
    #[validate(with = "validate_initials()")]
    initials: String,
    #[validate(with = "validate_length(2, 255)")]
    locality: String,
    #[validate(with = "validate_length(2, 16)")]
    postal_code: String,
    #[validate(with = "validate_length(1, 16)")]
    house_number: String,
    #[validate(with = "validate_length(1, 16)", optional)]
    house_number_addition: String,
    #[validate(with = "validate_length(2, 255)")]
    street_name: String,
    #[validate(csrf)]
    pub csrf_token: TokenValue,
}

impl WithCsrfToken for AuthorisedAgentForm {
    fn with_csrf_token(self, csrf_token: CsrfToken) -> Self {
        AuthorisedAgentForm {
            csrf_token: csrf_token.value,
            ..self
        }
    }
}

impl AuthorisedAgentForm {
    fn build_authorised_agent(
        validated: AuthorisedAgentFormValidated,
        current: Option<AuthorisedAgent>,
    ) -> AuthorisedAgent {
        if let Some(current) = current {
            AuthorisedAgent {
                last_name: validated.last_name,
                last_name_prefix: validated.last_name_prefix,
                initials: validated.initials,
                locality: validated.locality,
                postal_code: validated.postal_code,
                house_number: validated.house_number,
                house_number_addition: validated.house_number_addition,
                street_name: validated.street_name,
                ..current
            }
        } else {
            AuthorisedAgent {
                id: AuthorisedAgentId::new(),
                last_name: validated.last_name,
                last_name_prefix: validated.last_name_prefix,
                initials: validated.initials,
                locality: validated.locality,
                postal_code: validated.postal_code,
                house_number: validated.house_number,
                house_number_addition: validated.house_number_addition,
                street_name: validated.street_name,
                created_at: Utc::now(),
                updated_at: Utc::now(),
            }
        }
    }
}
