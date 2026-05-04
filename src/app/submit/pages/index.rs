use askama::Template;
use axum::response::IntoResponse;

use crate::{
    AppError, AppStore, Context, ElectoralDistrict, HtmlTemplate,
    candidate_lists::{CandidateList, CandidateListSummary},
    core::ModelLocale,
    filters,
    submit::IncompleteItems,
};

use super::SubmitPath;

struct SubmitCandidateList {
    list: CandidateList,
    download_h1_path_nl: String,
    download_h1_path_fry: String,
    download_h3_1_path_nl: String,
    download_h3_1_path_fry: String,
    download_h4_path_nl: String,
    download_h4_path_fry: String,
    download_h9_path_nl: String,
    download_h9_path_fry: String,
    download_eml_210_path_nl: String,
    download_eml_210_path_fry: String,
    person_count: usize,
    duplicate_districts: Vec<ElectoralDistrict>,
}

#[derive(Template)]
#[template(path = "submit/pages/index.html")]
pub struct IndexTemplate {
    candidate_lists: Vec<SubmitCandidateList>,
    incomplete_items: IncompleteItems,
}

pub async fn index(
    _: SubmitPath,
    context: Context,
    store: AppStore,
) -> Result<impl IntoResponse, AppError> {
    let candidate_lists = CandidateListSummary::list(&store)
        .into_iter()
        .map(|summary| {
            let person_count = summary.candidate_count();
            Ok(SubmitCandidateList {
                download_h1_path_nl: super::DownloadH1Path {
                    list_id: summary.list.id,
                    locale: ModelLocale::Nl,
                }
                .to_string(),
                download_h1_path_fry: super::DownloadH1Path {
                    list_id: summary.list.id,
                    locale: ModelLocale::Fry,
                }
                .to_string(),
                download_h3_1_path_nl: super::DownloadH31Path {
                    list_id: summary.list.id,
                    locale: ModelLocale::Nl,
                }
                .to_string(),
                download_h3_1_path_fry: super::DownloadH31Path {
                    list_id: summary.list.id,
                    locale: ModelLocale::Fry,
                }
                .to_string(),
                download_h4_path_nl: super::DownloadH4Path {
                    list_id: summary.list.id,
                    locale: ModelLocale::Nl,
                }
                .to_string(),
                download_h4_path_fry: super::DownloadH4Path {
                    list_id: summary.list.id,
                    locale: ModelLocale::Fry,
                }
                .to_string(),
                download_h9_path_nl: super::DownloadH9Path {
                    list_id: summary.list.id,
                    locale: ModelLocale::Nl,
                }
                .to_string(),
                download_h9_path_fry: super::DownloadH9Path {
                    list_id: summary.list.id,
                    locale: ModelLocale::Fry,
                }
                .to_string(),
                download_eml_210_path_nl: super::DownloadEml210Path {
                    list_id: summary.list.id,
                    locale: ModelLocale::Nl,
                }
                .to_string(),
                download_eml_210_path_fry: super::DownloadEml210Path {
                    list_id: summary.list.id,
                    locale: ModelLocale::Fry,
                }
                .to_string(),
                list: summary.list,
                person_count,
                duplicate_districts: summary.duplicate_districts,
            })
        })
        .collect::<Result<Vec<_>, AppError>>()?;

    Ok(HtmlTemplate(
        IndexTemplate {
            candidate_lists,
            incomplete_items: IncompleteItems::find_all(&store),
        },
        context,
    ))
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
    #[ignore] // TODO should pass again once #605, #607, and #608 have been implemented
    async fn index_shows_h1_downloads_for_complete_lists() -> Result<(), AppError> {
        let store = AppStore::new_for_test();
        let complete_list_id = CandidateListId::new();
        let incomplete_list_id = CandidateListId::new();
        let person_id = PersonId::new();

        sample_list_submitter(ListSubmitterId::new())
            .update(&store)
            .await?;
        sample_person(person_id).create(&store).await?;

        let mut complete_list = sample_candidate_list(complete_list_id);
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
