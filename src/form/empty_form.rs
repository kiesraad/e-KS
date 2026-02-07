use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use validate::Validate;

use crate::TokenValue;

#[derive(Clone, Default)]
pub struct EmptyFormValue {
    #[allow(unused)]
    pub created_at: DateTime<Utc>,
    #[allow(unused)]
    pub updated_at: DateTime<Utc>,
}

#[derive(Default, Serialize, Deserialize, Clone, Debug, Validate)]
#[validate(target = "EmptyFormValue")]
#[serde(default)]
pub struct EmptyForm {
    #[validate(csrf)]
    pub csrf_token: TokenValue,
}

#[cfg(test)]
impl EmptyForm {
    pub fn new(csrf_token: TokenValue) -> Self {
        EmptyForm { csrf_token }
    }
}
