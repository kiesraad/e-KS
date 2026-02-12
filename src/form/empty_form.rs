use serde::{Deserialize, Serialize};
use validate::Validate;

use crate::TokenValue;

#[derive(Clone, Default)]
pub struct EmptyFormValue;

#[derive(Default, Serialize, Deserialize, Clone, Debug, Validate)]
#[validate(target = "EmptyFormValue", timestamps = false)]
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
