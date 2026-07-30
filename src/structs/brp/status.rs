use serde::{Deserialize, Serialize};
use std::fmt::Display;

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub enum BrpStatus {
    #[default]
    NotStarted,
    InProgress,
    Aborted(String),
    Finished,
}

impl Display for BrpStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BrpStatus::NotStarted => write!(f, "not_started"),
            BrpStatus::InProgress => write!(f, "in_progress"),
            BrpStatus::Aborted(_) => write!(f, "aborted"),
            BrpStatus::Finished => write!(f, "finished"),
        }
    }
}
