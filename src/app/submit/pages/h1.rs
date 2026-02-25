use crate::{
    AppError, Config, Context, Store,
    candidate_lists::FullCandidateList,
    core::Pdf,
    submit::{H1, pages::DownloadH1Path},
};
use axum::{extract::State, response::IntoResponse};

pub async fn gen_h1(
    _: DownloadH1Path,
    list: FullCandidateList,
    State(store): State<Store>,
    State(config): State<Config>,
    context: Context,
) -> Result<impl IntoResponse, AppError> {
    let h1 = H1::new(&store, list, &context.election, context.locale)?;
    Ok(h1.generate(config.typst_url).await?)
}
