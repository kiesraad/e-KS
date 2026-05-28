use super::NameAuthorisationsPath;
use crate::{
    AppError, AppStore, Context, HtmlTemplate,
    app::list_designation::ListDesignation,
    common::Problematic,
    filters,
    list_submitters::ListSubmitter,
    name_authorisations::NameAuthorisation,
    political_groups::{PoliticalGroup, PoliticalGroupSteps},
};
use askama::Template;
use axum::response::IntoResponse;

#[derive(Template)]
#[template(path = "name_authorisations/pages/view.html")]
struct NameAuthorisationTemplate {
    name_authorisations: Vec<NameAuthorisation>,
    is_combined: bool,
    steps: PoliticalGroupSteps,
}

pub async fn list_name_authorisations(
    _: NameAuthorisationsPath,
    context: Context,
    store: AppStore,
) -> Result<impl IntoResponse, AppError> {
    let steps = PoliticalGroupSteps::new(&store)?;
    let is_combined =
        store.get_political_group().list_designation == Some(ListDesignation::Combined);
    Ok(HtmlTemplate(
        NameAuthorisationTemplate {
            name_authorisations: steps.name_authorisations.clone(),
            is_combined,
            steps,
        },
        context,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        AppError, AppStore, Context,
        name_authorisations::NameAuthorisationId,
        test_utils::{response_body_string, sample_name_authorisation},
    };
    use axum::{http::StatusCode, response::IntoResponse};

    #[tokio::test]
    async fn list_name_authorisations_shows_created_agent() -> Result<(), AppError> {
        let store = AppStore::new_for_test();

        let authorisation = sample_name_authorisation(NameAuthorisationId::new());
        authorisation.create(&store).await?;

        let response = list_name_authorisations(
            NameAuthorisationsPath {},
            Context::new_test_without_db(),
            store.clone(),
        )
        .await
        .unwrap()
        .into_response();

        assert_eq!(response.status(), StatusCode::OK);
        let body = response_body_string(response).await;
        assert!(body.contains(&authorisation.legal_name.to_string()));
        assert!(body.contains(authorisation.name.last_name.as_str()));

        Ok(())
    }

    #[tokio::test]
    async fn list_name_authorisations_shows_edit_link() -> Result<(), AppError> {
        let store = AppStore::new_for_test();

        let authorisation = sample_name_authorisation(NameAuthorisationId::new());
        authorisation.create(&store).await?;

        let response = list_name_authorisations(
            NameAuthorisationsPath {},
            Context::new_test_without_db(),
            store.clone(),
        )
        .await
        .unwrap()
        .into_response();

        assert_eq!(response.status(), StatusCode::OK);
        let body = response_body_string(response).await;
        assert!(body.contains(&authorisation.update_path().to_string()));

        Ok(())
    }
}
