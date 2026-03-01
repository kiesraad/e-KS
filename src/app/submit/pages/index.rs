use askama::Template;
use axum::response::IntoResponse;

use crate::{
    AppError, AppStore, Context, ElectoralDistrict, HtmlTemplate,
    candidate_lists::{CandidateList, CandidateListSummary, FullCandidateList},
    core::{AnyLocale, ModelLocale},
    filters,
    submit::H1,
};

use super::SubmitPath;

struct SubmitCandidateList {
    list: CandidateList,
    download_path_nl: String,
    download_path_fry: String,
    person_count: usize,
    duplicate_districts: Vec<ElectoralDistrict>,
    can_download: bool,
}

#[derive(Template)]
#[template(path = "submit/pages/index.html")]
pub struct IndexTemplate {
    candidate_lists: Vec<SubmitCandidateList>,
}

pub async fn index(
    _: SubmitPath,
    context: Context,
    store: AppStore,
) -> Result<impl IntoResponse, AppError> {
    let election = context.session.election;

    let candidate_lists = CandidateListSummary::list(&store)?
        .into_iter()
        .map(|summary| {
            let has_required_list_data =
                summary.person_count > 0 && !summary.list.electoral_districts.is_empty();
            let can_download = if has_required_list_data {
                let full_list = FullCandidateList::get(&store, summary.list.id)?;
                H1::new(&store, full_list, &election, ModelLocale::Nl).is_ok()
            } else {
                false
            };

            Ok(SubmitCandidateList {
                download_path_nl: super::DownloadH1Path {
                    list_id: summary.list.id,
                    locale: ModelLocale::Nl,
                }
                .to_string(),
                download_path_fry: super::DownloadH1Path {
                    list_id: summary.list.id,
                    locale: ModelLocale::Fry,
                }
                .to_string(),
                list: summary.list,
                person_count: summary.person_count,
                duplicate_districts: summary.duplicate_districts,
                can_download,
            })
        })
        .collect::<Result<Vec<_>, AppError>>()?;

    Ok(HtmlTemplate(IndexTemplate { candidate_lists }, context))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        AppStore, Context,
        candidate_lists::CandidateListId,
        list_submitters::ListSubmitterId,
        persons::PersonId,
        test_utils::{
            response_body_string, sample_candidate_list, sample_list_submitter, sample_person,
        },
    };
    use axum::response::IntoResponse;

    #[tokio::test]
    async fn index_shows_h1_downloads_for_complete_lists() -> Result<(), AppError> {
        let store = AppStore::new_for_test().await;
        let complete_list_id = CandidateListId::new();
        let incomplete_list_id = CandidateListId::new();
        let list_submitter_id = ListSubmitterId::new();
        let person_id = PersonId::new();

        sample_list_submitter(list_submitter_id)
            .create(&store)
            .await?;
        sample_person(person_id).create(&store).await?;

        let mut complete_list = sample_candidate_list(complete_list_id);
        complete_list.list_submitter_id = Some(list_submitter_id);
        complete_list.create(&store).await?;
        complete_list.append_candidate(&store, person_id).await?;

        let incomplete_list = sample_candidate_list(incomplete_list_id);
        incomplete_list.create(&store).await?;

        let response = index(SubmitPath, Context::new_test_without_db(), store)
            .await?
            .into_response();
        let body = response_body_string(response).await;

        assert!(
            body.contains(
                &super::super::DownloadH1Path {
                    list_id: complete_list_id,
                    locale: ModelLocale::Nl,
                }
                .to_string()
            )
        );
        assert!(
            !body.contains(
                &super::super::DownloadH1Path {
                    list_id: incomplete_list_id,
                    locale: ModelLocale::Nl,
                }
                .to_string()
            )
        );

        Ok(())
    }
}
