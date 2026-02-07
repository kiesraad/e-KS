use derive_more::{Deref, Display};
use serde::{Deserialize, Serialize};

#[derive(
    Debug, Deref, Clone, Copy, Display, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize,
)]
#[serde(transparent)]
pub struct UtcDateTime(chrono::DateTime<chrono::Utc>);

impl Default for UtcDateTime {
    fn default() -> Self {
        Self(chrono::Utc::now())
    }
}

impl From<chrono::DateTime<chrono::Utc>> for UtcDateTime {
    fn from(value: chrono::DateTime<chrono::Utc>) -> Self {
        Self(value)
    }
}

impl From<UtcDateTime> for chrono::DateTime<chrono::Utc> {
    fn from(value: UtcDateTime) -> Self {
        value.0
    }
}

impl UtcDateTime {
    pub fn now() -> Self {
        Self::default()
    }
}
