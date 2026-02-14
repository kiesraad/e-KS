use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct InitialQuery {
    #[serde(default)]
    initial: bool,
}

impl InitialQuery {
    pub fn is_initial(&self) -> bool {
        self.initial
    }

    pub fn should_warn(&self) -> bool {
        !self.initial
    }

    pub fn new() -> Self {
        Self { initial: true }
    }
}

impl Default for InitialQuery {
    fn default() -> Self {
        Self::new()
    }
}
