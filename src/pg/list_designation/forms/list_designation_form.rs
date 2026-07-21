use serde::Deserialize;
use validate::Validate;

use crate::list_designation::ListDesignation;

#[derive(Default, Debug, Clone)]
pub struct ListDesignationTarget {
    pub list_designation_type: ListDesignation,
}

impl From<Option<ListDesignation>> for ListDesignationTarget {
    fn from(value: Option<ListDesignation>) -> Self {
        Self {
            list_designation_type: value.unwrap_or_default(),
        }
    }
}

#[derive(Default, Deserialize, Debug, Validate)]
#[validate(target = "ListDesignationTarget")]
#[serde(default)]
pub struct ListDesignationForm {
    #[validate(parse = "ListDesignation")]
    pub list_designation_type: String,
}

impl From<Option<ListDesignation>> for ListDesignationForm {
    fn from(value: Option<ListDesignation>) -> Self {
        ListDesignationForm {
            list_designation_type: value.map(|d| d.to_string()).unwrap_or_default(),
        }
    }
}
