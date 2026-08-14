use askama::Template;
use axum::{
    extract::Query,
    response::{IntoResponse, Redirect, Response},
};

use crate::{
    AppError, Context, CsbContext, CsbEvent, CsbStore, HtmlTemplate, Overlay, QueryParamState,
    csb::examination::{
        CsbExaminationOverviewPath, extractors::CsbPoliticalGroup,
        paths::CsbPoliticalGroupDeletePath,
    },
    filters,
};

#[derive(Template)]
#[template(path = "csb/examination/pages/delete.html")]
struct CsbPoliticalGroupDeleteTemplate {
    political_group: CsbPoliticalGroup,
    overlay: Overlay,
    close_action: String,
}

pub async fn delete(
    _: CsbPoliticalGroupDeletePath,
    context: CsbContext,
    Query(query): Query<QueryParamState>,
    store: CsbStore,
) -> Result<Response, AppError> {
    let political_group = CsbPoliticalGroup::new_from_csb_store(&store);
    let close_action = political_group.examination_path().to_string();
    Ok(HtmlTemplate(
        CsbPoliticalGroupDeleteTemplate {
            close_action,
            political_group,
            overlay: Overlay::new(&query),
        },
        context,
    )
    .into_response())
}

pub async fn delete_submit(
    _: CsbPoliticalGroupDeletePath,
    store: CsbStore,
) -> Result<Response, AppError> {
    store.update(CsbEvent::Delete).await?;
    Ok(Redirect::to(&CsbExaminationOverviewPath.to_string()).into_response())
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use reqwest::StatusCode;

    use crate::{
        projection::WithCorrections, structs::common::Appellation, test_utils::response_body_string,
    };

    use super::*;

    #[tokio::test]
    async fn delete_page_includes_appellation_and_delete_button() -> Result<(), AppError> {
        let store = CsbStore::new_for_test();
        let context = CsbContext::new_test();

        let mut pg = store.get_political_group(WithCorrections::All);
        pg.appellation = Appellation::from_str("Test Partij").ok();
        store.set_political_group(pg);

        let response = delete(
            CsbPoliticalGroupDeletePath {
                stream_id: store.stream_id,
            },
            context,
            Query::default(),
            store,
        )
        .await?;

        assert_eq!(response.status(), StatusCode::OK);
        let body = response_body_string(response).await;
        assert!(body.contains("Delete Test Partij"));
        assert!(body.contains("<button type=\"submit\" class=\"button tertiary-destructive icon-trash\">Delete</button>"));

        Ok(())
    }
}
