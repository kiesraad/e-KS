use serde::{Deserialize, Serialize};
use validate::Validate;

use crate::{TokenValue, form::WithCsrfToken};

#[derive(Default, Serialize, Deserialize, Clone, Debug, Validate)]
#[validate(target = "()", build = "EmptyForm::post_validate")]
#[serde(default)]
pub struct EmptyForm {
    #[validate(csrf)]
    pub csrf_token: TokenValue,
}

impl EmptyForm {
    fn post_validate(_: EmptyFormValidated, _: Option<&()>) {
        // do nothing, we only need to validate the token
    }
}

impl From<TokenValue> for EmptyForm {
    fn from(csrf_token: TokenValue) -> Self {
        EmptyForm { csrf_token }
    }
}

impl WithCsrfToken for EmptyForm {
    fn with_csrf_token(self, csrf_token: crate::form::CsrfToken) -> Self {
        EmptyForm {
            csrf_token: csrf_token.value,
        }
    }
}
