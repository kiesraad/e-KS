use crate::{finalise::DocumentData, models::h1::H1};

impl From<&DocumentData> for H1 {
    fn from(data: &DocumentData) -> Self {
        Self {
            common: data.model_data.clone(),
            electoral_districts: data.electoral_districts.clone(),
            previously_seated: data.previously_seated,
            list_designation: data.list_designation,
            list_submitter: data.list_submitter.clone(),
            substitute_submitters: data.substitute_submitters.clone(),
        }
    }
}
