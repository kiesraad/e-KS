use crate::{
    AppError, AppStore, Config, Context,
    candidate_lists::FullCandidateList,
    candidates::Candidate,
    core::Pdf,
    submit::{H9, pages::DownloadH9Path},
};
use axum::{extract::State, response::IntoResponse};

pub async fn gen_h9(
    path: DownloadH9Path,
    list: FullCandidateList,
    candidate: Candidate,
    store: AppStore,
    State(config): State<Config>,
    context: Context,
) -> Result<impl IntoResponse, AppError> {
    let h9 = H9::new(
        &store,
        list,
        candidate,
        &context.session.election,
        path.locale,
    )?;

    h9.generate(config.typst_url).await
}
