use askama::Template;
use axum::response::IntoResponse;

use super::IndexPath;
use crate::{
    AppResponse, AppStore, Context, HtmlTemplate, candidate_lists::CandidateListSummary,
    common::Severity, filters, submit::AllProblems,
};

#[derive(Template)]
#[template(path = "common/pages/index.html")]
pub struct IndexTemplate {
    general_problems: usize,
    general_problems_severity: &'static str,
    problematic_lists: usize,
    problematic_lists_severity: &'static str,
}

pub async fn index(
    _: IndexPath,
    context: Context,
    store: AppStore,
) -> AppResponse<impl IntoResponse> {
    let political_group = store.get_political_group();
    let general_information_empty = political_group.is_general_information_empty(&store);

    let (general_problems, general_problems_severity) = if general_information_empty {
        (0, "")
    } else {
        let (general_problems, general_infos) = AllProblems::find_general_problems(&store);
        let problems = general_problems.flatten();
        let severity_class = if problems.is_empty() {
            (!general_infos.is_empty()).then_some(Severity::Info)
        } else {
            problems.iter().map(|p| p.severity()).max()
        }
        .map(|severity| severity.class())
        .unwrap_or("success");

        (problems.len() + general_infos.len(), severity_class)
    };
    // no infos, these are also not shown on the candidate list overview page
    let (list_problems, _) =
        AllProblems::find_list_problems(&CandidateListSummary::list(&store), &store)?;

    let (problematic_lists, problematic_lists_severity) = if list_problems.is_empty() {
        (0, "")
    } else {
        let count = list_problems.len();
        let severity_class = list_problems
            .into_iter()
            .flat_map(|p| p.problems)
            .map(|p| p.severity())
            .max()
            .map(|s| s.class())
            .unwrap_or_default();

        (count, severity_class)
    };

    Ok(HtmlTemplate(
        IndexTemplate {
            general_problems,
            general_problems_severity,
            problematic_lists,
            problematic_lists_severity,
        },
        context,
    ))
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
