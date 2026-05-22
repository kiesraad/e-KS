use serde::Deserialize;
use validate::Validate;

use crate::{TokenValue, list_designation::ListDesignation};

#[derive(Default, Debug, Clone)]
pub struct ListDesignationTarget {
    pub list_designation_type: ListDesignation,
}

impl From<ListDesignation> for ListDesignationTarget {
    fn from(value: ListDesignation) -> Self {
        Self {
            list_designation_type: value,
        }
    }
}

#[derive(Default, Deserialize, Debug, Validate)]
#[validate(target = "ListDesignationTarget")]
#[serde(default)]
pub struct ListDesignationForm {
    #[validate(parse = "ListDesignation")]
    pub list_designation_type: String,
    #[validate(csrf)]
    pub csrf_token: TokenValue,
}

impl From<ListDesignation> for ListDesignationForm {
    fn from(value: ListDesignation) -> Self {
        ListDesignationForm {
            list_designation_type: value.to_string(),
            csrf_token: Default::default(),
        }
    }
}
