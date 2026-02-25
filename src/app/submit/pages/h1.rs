use crate::{
    AppError, Context, Store,
    core::Pdf,
    submit::{H1, pages::DownloadH1Path},
};
use axum::{
    extract::State,
    http::{HeaderMap, HeaderValue, header},
    response::IntoResponse,
};

pub async fn gen_h1(
    DownloadH1Path { list_id }: DownloadH1Path,
    State(store): State<Store>,
    context: Context,
) -> Result<impl IntoResponse, AppError> {
    let h1 = H1::new(&store, list_id, context.election)?;
    let response = Pdf::H1(h1).generate().await?;

    let mut headers = HeaderMap::new();
    headers.insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static("application/pdf"),
    );
    headers.insert(
        header::CONTENT_DISPOSITION,
        HeaderValue::from_static(r#"attachment; filename="h1.pdf""#),
    );

    Ok((headers, axum::body::Body::from_stream(response)).into_response())
}
