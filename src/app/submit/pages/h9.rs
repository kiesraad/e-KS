use crate::{
    AppError, AppStore, Config, Context,
    candidate_lists::FullCandidateList,
    core::PdfZip,
    submit::{
        H9,
        pages::DownloadH9Path,
        structs::{electoral_districts::ElectoralDistricts, typst_candidate::ordered_candidates},
    },
};
use axum::{extract::State, response::IntoResponse};

pub async fn gen_h9(
    path: DownloadH9Path,
    list: FullCandidateList,
    store: AppStore,
    State(config): State<Config>,
    context: Context,
) -> Result<impl IntoResponse, AppError> {
    let mut h9s: Vec<H9> = vec![];
    let ordered_candidates = ordered_candidates(&mut list.candidates.clone(), path.locale)?;
    for candidate in list.candidates {
        let h9_model = H9::new(
            &store,
            &list.list,
            &ordered_candidates,
            candidate,
            &context.session.election,
            path.locale,
        )?;
        h9s.push(h9_model);
    }
    let district_name = if h9s.is_empty() {
        ElectoralDistricts::Some(vec![]).to_string()
    } else {
        h9s[0].electoral_districts.to_string()
    };
    dbg!(&district_name);
    PdfZip {
        filename: path.filename(district_name),
        pdfs: h9s,
    }
    .generate(config.typst_url)
    .await
}
