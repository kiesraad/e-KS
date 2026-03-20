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
            "candidate-list-export-{}.csv",
            candidate_list.districts_codes()
        ),
    }
    .generate_csv_response()
}

#[cfg(test)]
mod tests {
    use axum::body;
    use regex::Regex;
    use reqwest::{StatusCode, header};

    use crate::{
        candidate_lists::CandidateListId,
        persons::PersonId,
        test_utils::{sample_candidate_list, sample_person},
    };

    use super::*;

    #[tokio::test]
    async fn export_candidate_list_success() -> Result<(), AppError> {
        // setup
        let store = AppStore::new_for_test();

        let list_id = CandidateListId::new();
        let mut list = sample_candidate_list(list_id);

        let person_id1 = PersonId::new();
        let sample_person1 = sample_person(person_id1);
        sample_person1.create(&store).await?;
        list.candidates.push(person_id1);

        let person_id2 = PersonId::new();
        let mut sample_person2 = sample_person(person_id2);
        sample_person2.personal_data.bsn = None;
        sample_person2.create(&store).await?;
        list.candidates.push(person_id2);

        list.create(&store).await?;

        // test
        let response =
            export_candidate_list(CandidateListExportPath { list_id }, list, store).await?;

        // verify
        assert_eq!(response.status(), StatusCode::OK);
        let headers = response.headers();
        assert_eq!(
            headers
                .get(header::CONTENT_TYPE)
                .expect("content type header"),
            "text/csv"
        );

        let content_header = headers
            .get(header::CONTENT_DISPOSITION)
            .expect("content disposition header")
            .to_str()
            .unwrap();
        assert!(
            Regex::new("attachment; filename=\"candidate-list-export-(.{2}-)*(.{2})\\.csv\"")
                .unwrap()
                .is_match(content_header),
            "{}",
            format!("Actual: {}", content_header)
        );
        assert_eq!(
            headers
                .get(header::CACHE_CONTROL)
                .expect("cache control header"),
            "no-store, no-cache, must-revalidate, max-age=0"
        );
        assert_eq!(
            headers.get(header::PRAGMA).expect("pragma header"),
            "no-cache"
        );
        assert_eq!(headers.get(header::EXPIRES).expect("expires header"), "0");

        let expected_csv = include_str!("../testdata/candidates.csv");
        let body = String::from_utf8(
            body::to_bytes(response.into_body(), expected_csv.len() * 2)
                .await
                .unwrap()
                .to_vec(),
        )
        .unwrap();
        assert_eq!(body, expected_csv);

        Ok(())
    }
}
