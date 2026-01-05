use askama::Template;
use axum::response::Html;
use tracing::error;

use crate::{AppError, Context};

#[derive(Template)]
#[template(path = "index.html")]
pub struct IndexTemplate {}

pub async fn index() -> Result<Html<String>, AppError> {
    IndexTemplate {}.render().map(Html).map_err(|err| {
        error!(?err, "failed to render index template");
        AppError::InternalServerError
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::response::Html;

    #[tokio::test]
    async fn index_renders_html() {
        let Html(body) = index().await.unwrap();
        assert!(body.contains("Hello World!"));
    }
}
