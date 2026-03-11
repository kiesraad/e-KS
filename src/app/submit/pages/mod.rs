use axum::Router;
use axum_extra::routing::{RouterExt, TypedPath};
use serde::Deserialize;

use crate::{AppError, AppState, candidate_lists::CandidateListId, core::ModelLocale};

mod h1;
mod h9;
mod index;

#[derive(TypedPath, Deserialize)]
#[typed_path("/submit", rejection(AppError))]
pub struct SubmitPath;

#[derive(TypedPath, Deserialize)]
#[typed_path("/generate/{list_id}/{locale}/h1.pdf", rejection(AppError))]
pub struct DownloadH1Path {
    list_id: CandidateListId,
    locale: ModelLocale,
}

#[derive(TypedPath, Deserialize)]
#[typed_path("/generate/{list_id}/{locale}/h9.zip", rejection(AppError))]
pub struct DownloadH9Path {
    list_id: CandidateListId,
    locale: ModelLocale,
}

impl DownloadH9Path {
    pub fn filename(&self, district_name: String) -> String {
        format!("model-h9-{}-({}).zip", self.locale, district_name)
    }
}

pub fn router() -> Router<AppState> {
    Router::new()
        .typed_get(index::index)
        .typed_get(h1::gen_h1)
        .typed_get(h9::gen_h9)
}

#[cfg(test)]
mod tests {
    use axum::{
        Router,
        routing::{get, post},
    };
    use serde_json::Value;
    use tokio::{net::TcpListener, task::JoinHandle};

    use crate::Config;

    pub async fn setup_typst_webservice_stub() -> (JoinHandle<()>, Config) {
        let router = Router::new()
            .route(
                "/render-pdf/model-h1-nl.typ/{file_name}",
                get(|file_name: String| async { file_name }),
            )
            .route(
                "/render-pdf/batch",
                post(|body: String| async move {
                    let json: Value = serde_json::from_str(&body).unwrap();
                    json.as_array().unwrap().len().to_string()
                }),
            );
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        let server = tokio::spawn(async move {
            axum::serve(listener, router).await.unwrap();
        });

        let typst_url = Box::leak(format!("http://{addr}").into_boxed_str()).to_string();
        let config = Config {
            storage_url: "memory:".to_string(),
            typst_url,
        };

        (server, config)
    }
}
