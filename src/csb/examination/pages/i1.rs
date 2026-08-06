use axum::{extract::State, http::HeaderValue, response::IntoResponse};

use crate::{
    AppError, AppRequestState, CsbMainStore,
    core::{ModelLocale, constants::DEFAULT_DATE_FORMAT},
    csb::examination::{
        actions::{found_omissions, submitted_lists},
        pages::CsbI1DownloadPath,
    },
    models::{Pdf, i1::I1},
    utils::no_cache_headers,
};

const PDF_CONTENT_TYPE: &str = "application/pdf";

pub async fn gen_i1<S: AppRequestState>(
    _: CsbI1DownloadPath,
    main_store: CsbMainStore,
    State(state): State<S>,
) -> Result<impl IntoResponse, AppError> {
    let election = main_store.election;
    let registry = state.csb_store_registry();
    let submitted_lists = submitted_lists(registry, &election).await?;
    let found_omissions = found_omissions(registry, &election).await?;

    let model = I1 {
        election_name: election.formal_title(ModelLocale::Nl),
        election_date: election
            .election_date()
            .format(DEFAULT_DATE_FORMAT)
            .to_string(),
        session: election.public_session().into(),
        submitted_lists,
        found_omissions,
    };
    let filename = model.filename();
    let bytes = model.generate_bytes().await?;

    let headers = no_cache_headers::generate_attachment_headers(
        &filename,
        HeaderValue::from_static(PDF_CONTENT_TYPE),
    )?;

    Ok((headers, bytes).into_response())
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        http::{StatusCode, header},
        response::IntoResponse,
    };

    use crate::AppState;

    #[tokio::test]
    async fn gen_i1_returns_pdf_response() -> Result<(), AppError> {
        let main_store = CsbMainStore::new_for_test();
        let state = AppState::new_for_tests().await;
        let response = gen_i1(CsbI1DownloadPath, main_store, State(state))
            .await?
            .into_response();

        assert_eq!(response.status(), StatusCode::OK);
        let headers = response.headers();
        assert_eq!(
            headers.get(header::CONTENT_TYPE).expect("content type"),
            "application/pdf"
        );
        assert_eq!(
            headers
                .get(header::CONTENT_DISPOSITION)
                .expect("content disposition"),
            "attachment; filename=\"i1-proces-verbaal.pdf\""
        );
        assert_eq!(
            headers.get(header::CACHE_CONTROL).expect("cache control"),
            "no-store, no-cache, must-revalidate, max-age=0"
        );

        Ok(())
    }
}
