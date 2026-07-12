use crate::{
    finalise::DocumentData,
    models::{h9::H9, inputs::DetailedCandidate},
};

impl From<(&DocumentData, &DetailedCandidate)> for H9 {
    fn from((data, candidate): (&DocumentData, &DetailedCandidate)) -> Self {
        Self {
            common: data.model_data.clone(),
            electoral_districts: data.electoral_districts.clone(),
            detailed_candidate: candidate.clone(),
        }
    }
}
