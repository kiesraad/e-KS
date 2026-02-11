use serde::{Deserialize, Serialize};
use validate::Validate;

use crate::OptionStringExt;

use super::{Initials, LastName, LastNamePrefix};

#[derive(Default, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FullName {
    pub last_name: LastName,
    pub last_name_prefix: Option<LastNamePrefix>,
    pub initials: Initials,
}

impl FullName {
    /// Returns e.g. "van Dijk"
    pub fn last_name_with_prefix(&self) -> String {
        if let Some(prefix) = &self.last_name_prefix {
            format!("{} {}", prefix, self.last_name)
        } else {
            self.last_name.to_string()
        }
    }

    /// Returns e.g. "Dijk, van"
    pub fn last_name_with_prefix_appended(&self) -> String {
        if let Some(prefix) = &self.last_name_prefix {
            format!("{}, {}", self.last_name, prefix)
        } else {
            self.last_name.to_string()
        }
    }

    pub fn is_complete(&self) -> bool {
        !self.initials.is_empty() && !self.last_name.is_empty()
    }

    pub fn is_empty(&self) -> bool {
        self.initials.is_empty()
            && self.last_name.is_empty()
            && self.last_name_prefix.is_empty_or_none()
    }
}

#[derive(Default, Serialize, Deserialize, Clone, Debug, Validate)]
#[validate(target = "FullName", timestamps = false)]
pub struct FullNameForm {
    #[validate(parse = "LastName")]
    pub last_name: String,
    #[validate(parse = "LastNamePrefix", optional)]
    pub last_name_prefix: String,
    #[validate(parse = "Initials")]
    pub initials: String,
}

impl From<FullName> for FullNameForm {
    fn from(name: FullName) -> Self {
        FullNameForm {
            last_name: name.last_name.to_string(),
            last_name_prefix: name.last_name_prefix.to_string_or_default(),
            initials: name.initials.to_string(),
        }
    }
}
