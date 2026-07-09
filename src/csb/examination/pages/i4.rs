use axum::{extract::State, http::HeaderValue, response::IntoResponse};

use crate::{
    AppError, CsbMainStore, TypstRenderer,
    core::{ModelLocale, Pdf, constants::DEFAULT_DATE_FORMAT},
    csb::examination::pages::CsbI4DownloadPath,
    structs::typst::I4,
    utils::no_cache_headers,
};

const PDF_CONTENT_TYPE: &str = "application/pdf";

pub async fn gen_i4(
    _: CsbI4DownloadPath,
    main_store: CsbMainStore,
    State(renderer): State<TypstRenderer>,
) -> Result<impl IntoResponse, AppError> {
    let election = main_store.election;
    let model = I4 {
        election_name: election.formal_title(ModelLocale::Nl),
        election_date: election
            .election_date()
            .format(DEFAULT_DATE_FORMAT)
            .to_string(),
        public_session: election.public_session(),
        ..I4::default()
    };
    let filename = model.filename();
    let bytes = model.generate_bytes(&renderer).await?;

    let headers = no_cache_headers::generate_attachment_headers(
        &filename,
        HeaderValue::from_static(PDF_CONTENT_TYPE),
    )?;

    Ok((headers, bytes).into_response())
}

#[cfg(feature = "embed-typst")]
#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        http::{StatusCode, header},
        response::IntoResponse,
    };

    #[tokio::test]
    async fn gen_i4_returns_pdf_response() -> Result<(), AppError> {
        let main_store = CsbMainStore::new_for_test();

        let response = gen_i4(
            CsbI4DownloadPath,
            main_store,
            State(TypstRenderer::embedded(
                crate::utils::embed_typst::pdf_context(),
            )),
        )
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
            "attachment; filename=\"i4-geldigheid-en-nummering.pdf\""
        );
        assert_eq!(
            headers.get(header::CACHE_CONTROL).expect("cache control"),
            "no-store, no-cache, must-revalidate, max-age=0"
        );

        Ok(())
    }
}
