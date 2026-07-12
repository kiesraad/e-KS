use crate::{finalise::DocumentData, models::h4::H4};

impl From<&DocumentData> for H4 {
    fn from(data: &DocumentData) -> Self {
        Self {
            common: data.model_data.clone(),
        }
    }
}
