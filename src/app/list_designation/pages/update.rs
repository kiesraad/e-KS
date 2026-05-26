use askama::Template;
use axum::{
    extract::Query,
    response::{IntoResponse, Response},
};

use crate::{
    AppError, AppStore, Context, Form, HtmlTemplate, QueryParamState, filters,
    form::FormData,
    list_designation::{
        ListDesignation, forms::list_designation_form::ListDesignationForm,
        pages::ListDesignationUpdatePath,
    },
    list_submitters::ListSubmitter,
    name_authorisations::NameAuthorisation,
    political_groups::{PoliticalGroup, PoliticalGroupSteps},
};

#[derive(Template)]
#[template(path = "list_designation/pages/update.html")]
struct ListDesignationUpdateTemplate {
    form: FormData<ListDesignationForm>,
    steps: PoliticalGroupSteps,
}

pub async fn update_list_designation(
    _: ListDesignationUpdatePath,
    context: Context,
    store: AppStore,
    political_group: PoliticalGroup,
) -> Result<Response, AppError> {
    let steps = PoliticalGroupSteps::new(&store)?;
    Ok(HtmlTemplate(
        ListDesignationUpdateTemplate {
            steps,
            form: FormData::new_with_data(
                political_group.list_designation.into(),
                &context.session.csrf_token,
            ),
        },
        context,
    )
    .into_response())
}

pub async fn update_list_designation_submit(
    _: ListDesignationUpdatePath,
    context: Context,
    store: AppStore,
    mut political_group: PoliticalGroup,
    Query(query): Query<QueryParamState>,
    Form(form): Form<ListDesignationForm>,
) -> Result<Response, AppError> {
    let steps = PoliticalGroupSteps::new(&store)?;

    match form.validate_update(
        &political_group.list_designation.into(),
        &context.session.csrf_token,
    ) {
        Err(form_data) => Ok(HtmlTemplate(
            ListDesignationUpdateTemplate {
                form: form_data,
                steps,
            },
            context,
        )
        .into_response()),
        Ok(list_designation) => {
            political_group.list_designation = Some(list_designation.list_designation_type);
            political_group.update(&store).await?;

            Ok(query.redirect_or(PoliticalGroup::update_path()))
        }
    }
}
