use askama::Template;
use axum::response::IntoResponse;

use super::IndexPath;
use crate::{AppStore, Context, HtmlTemplate, filters, submit::Problems};

#[derive(Template)]
#[template(path = "common/pages/index.html")]
pub struct IndexTemplate {
    general_problems: usize,
    general_problems_severity: &'static str,
}

pub async fn index(_: IndexPath, context: Context, store: AppStore) -> impl IntoResponse {
    let problems = Problems::find_all(&store);
    let general_problems = problems.general.flatten();

    HtmlTemplate(
        IndexTemplate {
            general_problems: general_problems.len(),
            general_problems_severity: general_problems
                .iter()
                .map(|p| p.severity())
                .max()
                .map(|severity| severity.class())
                .unwrap_or("success"),
        },
        context,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::{ElectionConfig, core::AnyLocale, test_utils::response_body_string};

    #[tokio::test]
    async fn index_renders_html() {
        let body = index(
            IndexPath,
            Context::new_test_without_db(),
            AppStore::new_for_test(),
        )
        .await
        .into_response();
        let body = response_body_string(body).await;
        assert!(body.contains(ElectionConfig::EK27.title(AnyLocale::En)));
    }
}
