use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Default, Serialize, Deserialize)]
pub struct QueryParamState {
    #[serde(default)]
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    initial: bool,
    #[serde(default)]
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    highlight: Option<Uuid>,
}

impl QueryParamState {
    pub fn is_initial(&self) -> bool {
        self.initial
    }

    pub fn should_warn(&self) -> bool {
        !self.initial
    }

    pub fn get_highlight(&self) -> Option<Uuid> {
        self.highlight
    }

    pub fn is_success(&self) -> bool {
        self.success
    }

    pub fn created() -> Self {
        Self {
            initial: true,
            success: true,
            ..Default::default()
        }
    }

    pub fn success() -> Self {
        Self {
            success: true,
            ..Default::default()
        }
    }

    pub fn highlight(id: Uuid) -> Self {
        Self {
            highlight: Some(id),
            ..Default::default()
        }
    }

    pub fn highlight_success(id: Uuid) -> Self {
        Self {
            success: true,
            highlight: Some(id),
            ..Default::default()
        }
    }
}
