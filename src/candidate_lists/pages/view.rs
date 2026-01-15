use askama::Template;
use axum::response::IntoResponse;

use crate::{
    AppError, Context, DbConnection, HtmlTemplate,
    candidate_lists::{FullCandidateList, MAX_CANDIDATES},
    filters, t,
};

use super::{CandidateList, ViewCandidateListPath, load_candidate_list};

#[derive(Template)]
#[template(path = "candidate_lists/view.html")]
struct CandidateListViewTemplate {
    full_list: FullCandidateList,
    max_candidates: usize,
}

pub(crate) async fn view_candidate_list(
    ViewCandidateListPath { id }: ViewCandidateListPath,
    context: Context,
    DbConnection(mut conn): DbConnection,
) -> Result<impl IntoResponse, AppError> {
    let full_list = load_candidate_list(&mut conn, &id, context.locale).await?;

    Ok(HtmlTemplate(
        CandidateListViewTemplate {
            full_list,
            max_candidates: MAX_CANDIDATES,
        },
        context,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::response::IntoResponse;
    use sqlx::PgPool;
    use uuid::Uuid;

    use crate::{
        Context, DbConnection, Locale, candidate_lists, persons,
        test_utils::{response_body_string, sample_candidate_list, sample_person},
    };

    #[sqlx::test]
    async fn view_candidate_list_renders_candidates(pool: PgPool) -> Result<(), sqlx::Error> {
        let list_id = Uuid::new_v4();
        let list = sample_candidate_list(list_id);
        let person = sample_person(Uuid::new_v4());

        let mut conn = pool.acquire().await?;
        candidate_lists::repository::create_candidate_list(&mut conn, &list).await?;
        persons::repository::create_person(&mut conn, &person).await?;
        candidate_lists::repository::update_candidate_list_order(&mut conn, &list_id, &[person.id])
            .await?;

        let response = view_candidate_list(
            ViewCandidateListPath { id: list_id },
            Context::new(Locale::En),
            DbConnection(pool.acquire().await?),
        )
        .await
        .unwrap()
        .into_response();

        let body = response_body_string(response).await;
        assert!(body.contains("Jansen"));
        assert!(body.contains(&list.add_candidate_path()));

        Ok(())
    }
}
