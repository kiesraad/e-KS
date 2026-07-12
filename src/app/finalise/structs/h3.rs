use crate::{finalise::DocumentData, models::h3::H3};

impl From<&DocumentData> for H3 {
    fn from(data: &DocumentData) -> Self {
        Self {
            common: data.model_data.clone(),
            electoral_districts: data.electoral_districts.clone(),
            list_designation: data.list_designation,
            list_submitter: data.list_submitter.clone(),
            name_authorisations: data.name_authorisations.clone(),
        }
    }
}
