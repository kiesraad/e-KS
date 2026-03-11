use crate::{
    AppError, AppStore, Config, Context,
    candidate_lists::FullCandidateList,
    core::Pdf,
    submit::{H1, pages::DownloadH1Path},
};
use axum::{extract::State, response::IntoResponse};

pub async fn gen_h1(
    path: DownloadH1Path,
    list: FullCandidateList,
    store: AppStore,
    State(config): State<&Config>,
    context: Context,
) -> Result<impl IntoResponse, AppError> {
    let h1 = H1::new(&store, list, &context.session.election, path.locale)?;

    h1.generate(&config.typst_url).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        AppStore, Context,
        candidate_lists::CandidateListId,
        core::ModelLocale,
        list_submitters::ListSubmitterId,
        persons::PersonId,
        submit::pages::tests::setup_typst_webservice_stub,
        test_utils::{sample_candidate_list, sample_list_submitter, sample_person},
    };
    use axum::{
        http::{StatusCode, header},
        response::IntoResponse,
    };
    use regex::Regex;

    #[tokio::test]
    async fn gen_h1_missing_list_submitter_returns_error() -> Result<(), AppError> {
        let store = AppStore::new_for_test();
        let list_id = CandidateListId::new();
        let list = sample_candidate_list(list_id);
        list.create(&store).await?;

        let full_list = FullCandidateList::get(&store, list_id).expect("candidate list");
        let config = Config::new_test();

        let result = gen_h1(
            DownloadH1Path {
                list_id,
                locale: ModelLocale::Nl,
            },
            full_list,
            store,
            State(&config),
            Context::new_test_without_db(),
        )
        .await;

        match result {
            Err(AppError::IncompleteData(message)) => {
                assert_eq!(message, "Missing list submitter");
            }
            _ => panic!("expected missing list submitter error"),
        }

        Ok(())
    }

    #[cfg_attr(not(feature = "net-tests"), ignore = "requires network")]
    #[tokio::test]
    async fn gen_h1_returns_pdf_response() -> Result<(), AppError> {
        let store = AppStore::new_for_test();
        let list_id = CandidateListId::new();
        let list_submitter_id = ListSubmitterId::new();
        let person_id = PersonId::new();

        sample_list_submitter(list_submitter_id)
            .create(&store)
            .await?;
        sample_person(person_id).create(&store).await?;

        let mut list = sample_candidate_list(list_id);
        list.list_submitter_id = Some(list_submitter_id);
        list.create(&store).await?;
        list.append_candidate(&store, person_id).await?;

        let full_list = FullCandidateList::get(&store, list_id).expect("candidate list");

        let (server, config) = setup_typst_webservice_stub().await;

        let response = gen_h1(
            DownloadH1Path {
                list_id,
                locale: ModelLocale::Nl,
            },
            full_list,
            store,
            State(&config),
            Context::new_test_without_db(),
        )
        .await?
        .into_response();

        assert_eq!(response.status(), StatusCode::OK);
        let headers = response.headers();
        assert_eq!(
            headers
                .get(header::CONTENT_TYPE)
                .expect("content type header"),
            "application/pdf"
        );
        assert!(
            Regex::new("attachment; filename=\"model-h1-nl-\\((.{2}-)*(.{2})\\)\\.pdf\"")
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

        server.abort();

        Ok(())
    }
}
