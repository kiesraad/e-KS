use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct UtcDateTime(chrono::DateTime<chrono::Utc>);

impl std::ops::Deref for UtcDateTime {
    type Target = chrono::DateTime<chrono::Utc>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::fmt::Display for UtcDateTime {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

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
