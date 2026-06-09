use askama::Template;
use axum::response::IntoResponse;

use super::IndexPath;
use crate::{AppStore, Context, HtmlTemplate, common::Severity, filters, submit::Problems};

#[derive(Template)]
#[template(path = "common/pages/index.html")]
pub struct IndexTemplate {
    general_problems: usize,
    general_problems_severity: &'static str,
    problematic_lists: usize,
    problematic_lists_severity: &'static str,
}

pub async fn index(_: IndexPath, context: Context, store: AppStore) -> impl IntoResponse {
    let problems = Problems::find_all(&store);

    // TODO: refactor this after the problematic refactor
    let general_problems = problems.general.flatten(); // includes infos
    let list_problems = problems
        .lists
        .iter()
        .filter_map(|list| {
            let severity = list.problems.iter().map(|p| p.severity()).max()?;
            if severity > Severity::Info {
                Some(severity) // no infos, these are also not shown on the candidate list overview page
            } else {
                None
            }
        })
        .collect::<Vec<_>>();

    HtmlTemplate(
        IndexTemplate {
            general_problems: general_problems.len(),
            general_problems_severity: general_problems
                .iter()
                .map(|p| p.severity())
                .max()
                .map(|severity| severity.class())
                .unwrap_or("success"),
            problematic_lists: list_problems.len(),
            problematic_lists_severity: list_problems
                .iter()
                .max()
                .map(|severity| severity.class())
                .unwrap_or_default(),
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
