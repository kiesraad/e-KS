use askama::Template;
use axum::{
    extract::Query,
    response::{IntoResponse, Response},
};

use crate::{
    AppError, AppStore, Context, HtmlTemplate, QueryParamState, filters,
    form::{Form, FormData},
    list_designation::ListDesignation,
    list_submitters::ListSubmitter,
    name_authorisations::NameAuthorisation,
    political_groups::{PoliticalGroup, PoliticalGroupForm, PoliticalGroupSteps},
};

use super::PoliticalGroupUpdatePath;

#[derive(Template)]
#[template(path = "app/political_groups/pages/update.html")]
struct PoliticalGroupUpdateTemplate {
    form: FormData<PoliticalGroupForm>,
    steps: PoliticalGroupSteps,
}

pub async fn update_political_group(
    _: PoliticalGroupUpdatePath,
    context: Context,
    store: AppStore,
    political_group: PoliticalGroup,
    Query(query): Query<QueryParamState>,
) -> Result<Response, AppError> {
    let steps = PoliticalGroupSteps::new(&store, query.is_initial())?;

    Ok(HtmlTemplate(
        PoliticalGroupUpdateTemplate {
            form: FormData::new_with_data(political_group.clone().into()),
            steps,
        },
        context,
    )
    .into_response())
}

pub async fn update_political_group_submit(
    _: PoliticalGroupUpdatePath,
    context: Context,
    political_group: PoliticalGroup,
    store: AppStore,
    Query(query): Query<QueryParamState>,
    Form(form): Form<PoliticalGroupForm>,
) -> Result<Response, AppError> {
    let steps = PoliticalGroupSteps::new(&store, query.is_initial())?;

    match form.validate_update(&political_group) {
        Err(form_data) => Ok(HtmlTemplate(
            PoliticalGroupUpdateTemplate {
                form: form_data,
                steps,
            },
            context,
        )
        .into_response()),
        Ok(political_group) => {
            political_group.update(&store).await?;

            Ok(query.redirect_or_preserving_initial(NameAuthorisation::list_path()))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        AppError, AppStore, Context, Form, QueryParamState,
        common::{DisplayName, PreviousElectionResults},
        name_authorisations::NameAuthorisationId,
        test_utils::{
            response_body_string, sample_name_authorisation, sample_political_group_form,
        },
    };
    use axum::{
        http::{StatusCode, header},
        response::IntoResponse,
    };
    use axum_extra::routing::TypedPath;

    #[tokio::test]
    async fn update_political_group_renders_existing_data() -> Result<(), AppError> {
        let store = AppStore::new_for_test();
        let political_group = store.get_political_group();

        let response = update_political_group(
            PoliticalGroupUpdatePath {},
            Context::new_test_without_db(),
            store,
            political_group,
            Query(QueryParamState::default()),
        )
        .await
        .unwrap()
        .into_response();

        assert_eq!(response.status(), StatusCode::OK);
        let body = response_body_string(response).await;
        assert!(body.contains("name=\"csrf_token\""));
        assert!(body.contains("Kiesraad Demo"));

        Ok(())
    }

    /// In paper-corrections mode the app layout shows the warning banner
    /// (with the group name and an exit form) and hides the finalise menu.
    #[tokio::test]
    async fn renders_corrections_banner_and_hides_finalise_in_corrections_mode()
    -> Result<(), AppError> {
        let csb_store = crate::CsbStore::new_for_test();
        csb_store.set_political_group(crate::test_utils::sample_political_group());
        let store = AppStore::paper_corrections(csb_store.clone());
        let context = Context::new(
            &store,
            crate::Session::new_test_with_locale(crate::Locale::En),
        );
        let political_group = store.get_political_group();

        let response = update_political_group(
            PoliticalGroupUpdatePath {},
            context,
            store,
            political_group,
            Query(QueryParamState::default()),
        )
        .await
        .unwrap()
        .into_response();

        assert_eq!(response.status(), StatusCode::OK);
        let body = response_body_string(response).await;
        assert!(body.contains("paper-corrections-banner"));
        // The banner names the group being corrected and warns to only enter
        // handwritten corrections from the handed-in paper documents.
        assert!(body.contains("You are correcting Kiesraad Demo."));
        assert!(body.contains("handed-in paper documents"));
        // Leaving corrections mode posts to the stop route of the CSB stream.
        assert!(body.contains(&format!(
            "/csb/examination/{}/paper-corrections/stop",
            csb_store.stream_id
        )));
        // The finalise menu item is hidden.
        assert!(!body.contains("/finalise"));

        Ok(())
    }

    #[tokio::test]
    async fn update_political_group_persists_and_redirects() -> Result<(), AppError> {
        let store = AppStore::new_for_test();
        let political_group = store.get_political_group();

        sample_name_authorisation(NameAuthorisationId::new())
            .create(&store)
            .await?;

        let context = Context::new_test_without_db();
        let mut form = sample_political_group_form();
        form.display_name = "a".repeat(DisplayName::MAX_CHAR_COUNT); // max length

        let response = update_political_group_submit(
            PoliticalGroupUpdatePath {},
            context,
            political_group,
            store.clone(),
            Query(QueryParamState::default()),
            Form(form),
        )
        .await
        .unwrap();

        assert_eq!(response.status(), StatusCode::SEE_OTHER);
        let location = response
            .headers()
            .get(header::LOCATION)
            .expect("location header")
            .to_str()
            .expect("location header value");
        assert_eq!(
            location,
            NameAuthorisation::list_path()
                .with_query_params(QueryParamState::success())
                .to_string()
        );

        let updated = store.get_political_group();
        assert_eq!(
            updated.previous_election_results,
            Some(PreviousElectionResults::OneToFifteenSeats)
        );
        assert_eq!(
            updated.display_name.as_deref().map(|v| v.to_string()),
            Some("a".repeat(DisplayName::MAX_CHAR_COUNT))
        );

        Ok(())
    }

    #[tokio::test]
    async fn update_political_group_invalid_form_renders_template() -> Result<(), AppError> {
        let store = AppStore::new_for_test();

        sample_name_authorisation(NameAuthorisationId::new())
            .create(&store)
            .await?;

        let context = Context::new_test_without_db();
        let mut form = sample_political_group_form();

        form.display_name = "a".repeat(DisplayName::MAX_CHAR_COUNT + 1); // Invalid value (too long)

        let response = update_political_group_submit(
            PoliticalGroupUpdatePath {},
            context,
            store.get_political_group(),
            store,
            Query(QueryParamState::default()),
            Form(form),
        )
        .await
        .unwrap()
        .into_response();

        assert_eq!(response.status(), StatusCode::OK);
        let body = response_body_string(response).await;
        assert!(body.contains("The value is too long"));

        Ok(())
    }
}
