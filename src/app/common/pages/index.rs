use askama::Template;
use axum::response::IntoResponse;

use super::IndexPath;
use axum_extra::routing::TypedPath;

use crate::{
    AppResponse, AppStore, Context, HtmlTemplate, QueryParamState,
    candidate_lists::CandidateListSummary, common::Severity, filters,
    list_designation::ListDesignation, submit::AllProblems,
};

#[derive(Template)]
#[template(path = "common/pages/index.html")]
pub struct IndexTemplate {
    general_problems: usize,
    general_problems_severity: &'static str,
    general_information_path: String,
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

    let general_information_path = if general_information_empty {
        ListDesignation::update_path()
            .with_query_params(QueryParamState::initial())
            .to_string()
    } else {
        ListDesignation::update_path().to_string()
    };

    Ok(HtmlTemplate(
        IndexTemplate {
            general_problems,
            general_problems_severity,
            general_information_path,
            problematic_lists,
            problematic_lists_severity,
        },
        context,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    use axum_extra::routing::TypedPath;

    use crate::{
        ElectionConfig, QueryParamState,
        core::AnyLocale,
        list_designation::ListDesignation,
        political_groups::PoliticalGroup,
        test_utils::{response_body_string, sample_political_group},
    };

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

    fn general_information_card_link(initial: bool) -> String {
        format!(
            r#"<a class="card" href="{}">"#,
            if initial {
                ListDesignation::update_path()
                    .with_query_params(QueryParamState::initial())
                    .to_string()
                    .replace('&', "&#38;") // Askama HTML-escapes `&` to `&#38;` inside attribute values
            } else {
                ListDesignation::update_path().to_string()
            }
        )
    }

    #[tokio::test]
    async fn general_information_link_has_initial_when_empty() {
        let store = AppStore::new_for_test();

        // Reset to an empty political group
        PoliticalGroup::default().update(&store).await.unwrap();
        let pg = store.get_political_group();
        assert!(pg.is_general_information_empty(&store));

        let body = index(IndexPath, Context::new_test_without_db(), store)
            .await
            .into_response();
        let body = response_body_string(body).await;

        assert!(body.contains(&general_information_card_link(true)));
        assert!(!body.contains(&general_information_card_link(false)));
    }

    #[tokio::test]
    async fn general_information_link_has_no_initial_when_not_empty() {
        let store = AppStore::new_for_test();

        // Sample political group with filled in values
        sample_political_group().update(&store).await.unwrap();
        let pg = store.get_political_group();
        assert!(!pg.is_general_information_empty(&store));

        let body = index(IndexPath, Context::new_test_without_db(), store)
            .await
            .into_response();
        let body = response_body_string(body).await;

        assert!(!body.contains(&general_information_card_link(true)));
        assert!(body.contains(&general_information_card_link(false)));
    }
}
