use axum::response::Response;

use crate::{
    AppError, AppStore,
    candidate_lists::{CandidateList, pages::CandidateListExportPath, structs::CandidateRecord},
    core::Csv,
};

pub async fn export_candidate_list(
    _: CandidateListExportPath,
    candidate_list: CandidateList,
    store: AppStore,
) -> Result<Response, AppError> {
    let mut records: Vec<CandidateRecord> = vec![];
    for person_id in &candidate_list.candidates {
        records.push(store.get_person(*person_id)?.into());
    }
    Csv {
        records,
        filename: format!(
            "candidate-list-export-({}).csv",
            candidate_list.districts_codes()
        ),
    }
    .generate_csv_response()
}
