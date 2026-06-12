use askama::Template;
use axum::{
    extract::Query,
    response::{IntoResponse, Response},
};

use crate::{
    AppError, AppStore, Context, Form, HtmlTemplate, Overlay, QueryParamState, filters,
    form::FormData,
    list_submitters::{ListSubmitter, ListSubmitterData, ListSubmitterForm},
};

use super::ListSubmitterUpdatePath;

#[derive(Template)]
#[template(path = "list_submitters/pages/update.html")]
struct ListSubmitterUpdateTemplate {
    form: FormData<ListSubmitterForm>,
    should_warn: bool,
    overlay: Overlay,
}

pub async fn update_list_submitter(
    _: ListSubmitterUpdatePath,
    context: Context,
    store: AppStore,
    Query(query): Query<QueryParamState>,
) -> Result<Response, AppError> {
    let list_submitter = store.get_list_submitter();
    let should_warn = !list_submitter.is_empty();
    Ok(HtmlTemplate(
        ListSubmitterUpdateTemplate {
            form: FormData::new_with_data(list_submitter.into(), &context.session.csrf_token),
            should_warn,
            overlay: Overlay::new(&query),
        },
        context,
    )
    .into_response())
}

pub async fn update_list_submitter_submit(
    _: ListSubmitterUpdatePath,
    context: Context,
    store: AppStore,
    Query(query): Query<QueryParamState>,
    Form(form): Form<ListSubmitterForm>,
) -> Result<Response, AppError> {
    let list_submitter = store.get_list_submitter();
    match form.validate_update_with_checks(
        &ListSubmitterData::from(list_submitter.clone()),
        &context.session.csrf_token,
    ) {
        Err(form_data) => Ok(HtmlTemplate(
            ListSubmitterUpdateTemplate {
                form: *form_data,
                should_warn: true,
                overlay: Overlay::new(&query),
            },
            context,
        )
        .into_response()),

        Ok(list_submitter_data) => {
            let mut updated = ListSubmitter {
                id: list_submitter.id,
                ..list_submitter_data.into()
            };
            updated.address.update_is_known_in_bag();

            updated.update(&store).await?;

            Ok(query.redirect_or(ListSubmitter::view_path()))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        AppError, AppStore, Context, Form, QueryParamState,
        common::{Address, PotentialProblems, Problematic},
        test_utils::{response_body_string, sample_list_submitter, sample_list_submitter_form},
    };
    use axum::{
        http::{StatusCode, header},
        response::IntoResponse,
    };
    use axum_extra::routing::TypedPath;

    #[tokio::test]
    async fn update_list_submitter_renders_existing_submitter() -> Result<(), AppError> {
        let store = AppStore::new_for_test();

        let list_submitter = sample_list_submitter(crate::list_submitters::ListSubmitterId::new());
        list_submitter.update(&store).await?;

        let response = update_list_submitter(
            ListSubmitterUpdatePath {},
            Context::new_test_without_db(),
            store,
            Query(QueryParamState::default()),
        )
        .await
        .unwrap()
        .into_response();

        assert_eq!(response.status(), StatusCode::OK);
        let body = response_body_string(response).await;
        assert!(body.contains(list_submitter.name.last_name.as_str()));

        Ok(())
    }

    #[tokio::test]
    async fn update_list_submitter_persists_and_redirects() -> Result<(), AppError> {
        let store = AppStore::new_for_test();

        let context = Context::new_test_without_db();
        let csrf_token = context.session.csrf_token.clone();
        let mut form = sample_list_submitter_form(&csrf_token);
        form.name.last_name = "Updated".to_string();

        let response = update_list_submitter_submit(
            ListSubmitterUpdatePath {},
            context,
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
            ListSubmitter::view_path()
                .with_query_params(QueryParamState::success())
                .to_string()
        );

        let updated = store.get_list_submitter();
        assert_eq!(updated.name.last_name.to_string(), "Updated");

        Ok(())
    }

    #[tokio::test]
    async fn update_list_submitter_flags_dutch_address_unknown_in_bag() -> Result<(), AppError> {
        let store = AppStore::new_for_test();

        let context = Context::new_test_without_db();
        let csrf_token = context.session.csrf_token.clone();
        let mut form = sample_list_submitter_form(&csrf_token);
        // a Dutch address (no country) that does not exist in the BAG
        form.address.street_name = "Nepstraat".to_string();
        form.address.house_number = "1".to_string();
        form.address.house_number_addition = String::new();
        form.address.postal_code = "1234 AB".to_string();
        form.address.locality = "Juinen".to_string();

        update_list_submitter_submit(
            ListSubmitterUpdatePath {},
            context,
            store.clone(),
            Query(QueryParamState::default()),
            Form(form),
        )
        .await
        .unwrap();

        // the handler runs the BAG lookup and persists the outcome...
        let stored = store.get_list_submitter();
        assert!(matches!(
            &stored.address,
            Address::Dutch(address) if address.known_in_bag == Some(false)
        ));
        // ...which surfaces as an `UnknownAddress` warning.
        assert!(
            stored
                .get_problems(())
                .potential_problems
                .contains(&PotentialProblems::UnknownAddress)
        );

        Ok(())
    }

    #[tokio::test]
    async fn update_list_submitter_invalid_form_renders_template() -> Result<(), AppError> {
        let store = AppStore::new_for_test();

        let context = Context::new_test_without_db();
        let csrf_token = context.session.csrf_token.clone();
        let mut form = sample_list_submitter_form(&csrf_token);
        form.name.last_name = " ".to_string();

        let response = update_list_submitter_submit(
            ListSubmitterUpdatePath {},
            context,
            store,
            Query(QueryParamState::default()),
            Form(form),
        )
        .await
        .unwrap()
        .into_response();

        assert_eq!(response.status(), StatusCode::OK);
        let body = response_body_string(response).await;
        assert!(body.contains("This field must not be empty."));

        Ok(())
    }
}
