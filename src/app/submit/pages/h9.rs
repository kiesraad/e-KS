use crate::{
    AppError, AppStore, Config, Context,
    candidate_lists::FullCandidateList,
    core::PdfZip,
    submit::{H9, pages::DownloadH9Path, structs::typst_candidate::ordered_candidates},
};
use axum::{extract::State, response::IntoResponse};

pub async fn gen_h9(
    path: DownloadH9Path,
    list: FullCandidateList,
    store: AppStore,
    State(config): State<&Config>,
    context: Context,
) -> Result<impl IntoResponse, AppError> {
    // front load the ordering and Typst conversion of candidates
    // so we only need to do it once for all H9 models
    let ordered_candidates = ordered_candidates(&mut list.candidates.clone(), path.locale)?;

    let mut h9s = vec![];
    for candidate in list.candidates {
        let h9_model = H9::new(
            &store,
            &list.list,
            &ordered_candidates,
            candidate,
            &context.session.election,
            path.locale,
        );
        h9s.push(h9_model?);
    }

    let filename = if list.list.contains_all_districts(&context.session.election) {
        "model-h9.zip".to_string()
    } else {
        format!("model-h9-{}.zip", list.list.districts_codes())
    };

    PdfZip {
        filename,
        pdfs: h9s,
    }
    .generate(&config.typst_url)
    .await
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        AppStore, Context,
        candidate_lists::CandidateListId,
        candidates::Candidate,
        core::ModelLocale,
        persons::PersonId,
        submit::pages::tests::setup_typst_webservice_stub,
        test_utils::{sample_candidate_list, sample_person},
    };
    use axum::{
        body,
        http::{StatusCode, header},
        response::IntoResponse,
    };
    use regex::Regex;

    #[tokio::test]
    async fn gen_h9_missing_designation_error() -> Result<(), AppError> {
        // setup
        let store = AppStore::new_for_test();

        let mut political_group = store.get_political_group();
        political_group.display_name = None;
        political_group.legal_name = None;
        political_group.update(&store).await?;

        let list_id = CandidateListId::new();
        let mut list = sample_candidate_list(list_id);

        let person_id = PersonId::new();
        let sample_person = sample_person(person_id);
        sample_person.create(&store).await?;

        list.candidates.push(person_id);
        list.create(&store).await?;

        let full_list = FullCandidateList {
            list: sample_candidate_list(list_id),
            candidates: vec![Candidate {
                list_id,
                position: 1,
                person: sample_person,
            }],
        };
        let config = Config::new_test();

        // test
        let result = gen_h9(
            DownloadH9Path {
                list_id,
                locale: ModelLocale::Nl,
            },
            full_list,
            store,
            State(&config),
            Context::new_test_without_db(),
        )
        .await;

        // verify
        match result {
            Err(err) => {
                assert!(matches!(err, AppError::IncompleteData(_)));
            }
            _ => {
                panic!("expected missing data error")
            }
        }
        Ok(())
    }

    #[cfg_attr(not(feature = "net-tests"), ignore = "requires network")]
    #[tokio::test]
    async fn gen_h9_returns_zip_response() -> Result<(), AppError> {
        // setup
        let store = AppStore::new_for_test();

        let list_id = CandidateListId::new();
        let mut list = sample_candidate_list(list_id);

        let person_id1 = PersonId::new();
        let sample_person1 = sample_person(person_id1);
        sample_person1.create(&store).await?;
        list.candidates.push(person_id1);

        let person_id2 = PersonId::new();
        let sample_person2 = sample_person(person_id2);
        sample_person2.create(&store).await?;
        list.candidates.push(person_id2);

        list.create(&store).await?;

        let full_list = FullCandidateList {
            list: sample_candidate_list(list_id),
            candidates: vec![
                Candidate {
                    list_id,
                    position: 1,
                    person: sample_person1,
                },
                Candidate {
                    list_id,
                    position: 2,
                    person: sample_person2,
                },
            ],
        };

        let (server, config) = setup_typst_webservice_stub().await;

        // test
        let response = gen_h9(
            DownloadH9Path {
                list_id,
                locale: ModelLocale::Nl,
            },
            full_list,
            store,
            State(&config),
            Context::new_test_without_db(),
        )
        .await
        .into_response();

        // verify
        assert_eq!(response.status(), StatusCode::OK);
        let headers = response.headers();
        assert_eq!(
            headers
                .get(header::CONTENT_TYPE)
                .expect("content type header"),
            "application/zip"
        );
        assert!(
            Regex::new("attachment; filename=\"model-h9-(.{2}-)*(.{2})\\.zip\"")
                .unwrap()
                .is_match(
                    headers
                        .get(header::CONTENT_DISPOSITION)
                        .expect("content disposition header")
                        .to_str()
                        .unwrap()
                )
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
        let body = String::from_utf8(
            body::to_bytes(response.into_body(), 1024)
                .await
                .unwrap()
                .to_vec(),
        )
        .unwrap();
        // the stub returns the number of expected pdfs in the zip (2 candidates = 2 pdfs)
        assert_eq!(body, "2");

        server.abort();

        Ok(())
    }
}
