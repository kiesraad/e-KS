use axum::{extract::State, response::IntoResponse};

use crate::{
    AppError, AppEvent, Config, RequestCtx,
    core::Pdf,
    submit::{pages::DownloadH4Path, structs::h4::H4},
};

pub async fn gen_h4(
    path @ DownloadH4Path { list_id, locale }: DownloadH4Path,
    State(config): State<&Config>,
    ctx: RequestCtx,
) -> Result<impl IntoResponse, AppError> {
    let h4 = H4::new(&ctx.store, list_id, ctx.election(), locale)?;

    ctx.store
        .update(AppEvent::DownloadFile {
            file_name: h4.filename().to_string(),
            download_path: path.to_string(),
            list_id,
        })
        .await?;

    h4.generate(&config.typst_url).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        AppStore, Context, QueryParamState, RequestCtx, candidate_lists::CandidateListId,
        core::ModelLocale, submit::pages::tests::setup_typst_webservice_stub,
        test_utils::sample_candidate_list,
    };
    use axum::{
        http::{StatusCode, header},
        response::IntoResponse,
    };
    use regex::Regex;

    #[tokio::test]
    async fn gen_h4_missing_designation_returns_error() -> Result<(), AppError> {
        let store = AppStore::new_for_test();

        let mut political_group = store.get_political_group();
        political_group.display_name = None;
        political_group.legal_name = None;
        political_group.update(&store).await?;

        let list_id = CandidateListId::new();
        let list = sample_candidate_list(list_id);
        list.create(&store).await?;

        let config = Config::new_test();

        let result = gen_h4(
            DownloadH4Path {
                list_id,
                locale: ModelLocale::Nl,
            },
            State(&config),
            RequestCtx {
                context: Context::new_test_without_db(),
                store,
                query: QueryParamState::default(),
            },
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
    async fn gen_h1_returns_pdf_response() -> Result<(), AppError> {
        let store = AppStore::new_for_test();
        let list_id = CandidateListId::new();

        let list = sample_candidate_list(list_id);
        list.create(&store).await?;

        let (server, config) = setup_typst_webservice_stub().await;

        let response = gen_h4(
            DownloadH4Path {
                list_id,
                locale: ModelLocale::Nl,
            },
            State(&config),
            RequestCtx {
                context: Context::new_test_without_db(),
                store,
                query: QueryParamState::default(),
            },
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
            Regex::new("attachment; filename=\"model-h4-(\\(.+\\))?.pdf\"")
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
