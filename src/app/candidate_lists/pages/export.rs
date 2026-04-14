use axum::response::Response;

use crate::{
    AppError, AppEvent, AppStore,
    candidate_lists::{
        FullCandidateList,
        pages::CandidateListExportPath,
        structs::{CSV_HEADERS, CandidateRecordCsv},
    },
    core::Csv,
};

pub async fn export_candidate_list(
    _: CandidateListExportPath,
    full_list: FullCandidateList,
    store: AppStore,
) -> Result<Response, AppError> {
    let list_id = full_list.list.id;
    let short_id = &list_id.to_string()[..8];
    let file_name = format!("{short_id}-{}.csv", full_list.list.districts_codes());
    let records = full_list
        .candidates
        .into_iter()
        .map(|candidate| CandidateRecordCsv::from(candidate.person))
        .collect::<Vec<_>>();

    let csv = Csv {
        records,
        filename: file_name.clone(),
        headers: Some(CSV_HEADERS.to_vec()),
    };

    let (response, file_size) = csv.generate_csv_response()?;

    store
        .update(AppEvent::ExportCsv {
            file_name,
            file_size,
            list_id,
        })
        .await?;

    Ok(response)
}

#[cfg(test)]
mod tests {
    use axum::body;
    use regex::Regex;
    use reqwest::{StatusCode, header};

    use crate::{
        AppStore,
        candidate_lists::CandidateListId,
        persons::PersonId,
        test_utils::{sample_candidate_list, sample_person},
    };

    use super::*;

    const CSV_HEADER: &str = include_str!("../testdata/csv_header.csv");

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

        let full_list = FullCandidateList::get(&store, list_id)?;

        // test
        let response =
            export_candidate_list(CandidateListExportPath { list_id }, full_list, store).await?;

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
            Regex::new("attachment; filename=\"[0-9a-f]{8}-(.{2,}-)*(.{2,})\\.csv\"")
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

    #[tokio::test]
    async fn export_candidate_list_includes_header_without_candidates() -> Result<(), AppError> {
        let store = AppStore::new_for_test();

        let list_id = CandidateListId::new();
        let list = sample_candidate_list(list_id);
        list.create(&store).await?;

        let full_list = FullCandidateList::get(&store, list_id)?;

        let response =
            export_candidate_list(CandidateListExportPath { list_id }, full_list, store).await?;

        assert_eq!(response.status(), StatusCode::OK);

        let expected_csv = format!(
            "{}\n",
            CSV_HEADER.trim_end_matches('\n').trim_end_matches('\r')
        );
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
