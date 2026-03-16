use serde::{Deserialize, Serialize};

use crate::OptionAsStrExt;

use super::{FirstName, Initials, LastName, LastNamePrefix};

#[derive(Default, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FullName {
    pub first_name: Option<FirstName>,
    pub last_name: LastName,
    pub last_name_prefix: Option<LastNamePrefix>,
    pub initials: Initials,
}

impl FullName {
    /// Returns e.g. "van Dijk, A.B. (Anne)"
    pub fn display(&self) -> String {
        if let Some(first_name) = &self.first_name {
            format!(
                "{}, {} ({})",
                self.last_name_with_prefix(),
                self.initials,
                first_name
            )
        } else {
            format!("{}, {}", self.last_name_with_prefix(), self.initials)
        }
    }

    pub fn initials_with_first_name(&self) -> String {
        if let Some(first_name) = &self.first_name {
            format!("{} ({})", self.initials, first_name)
        } else {
            self.initials.to_string()
        }
    }

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

impl PartialOrd for FullName {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for FullName {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.last_name
            .cmp(&other.last_name)
            .then_with(|| self.last_name_prefix.cmp(&other.last_name_prefix))
            .then_with(|| self.initials.cmp(&other.initials))
    }
}
