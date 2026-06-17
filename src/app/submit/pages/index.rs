use askama::Template;
use axum::response::IntoResponse;

use crate::{
    AppError, AppStore, Context, HtmlTemplate,
    common::{HasSeverity, Severity},
    core::ModelLocale,
    filters,
    list_submitters::ListSubmitter,
    submit::AllProblems,
};

use super::SubmitPath;

#[derive(Template)]
#[template(path = "submit/pages/index.html")]
pub struct IndexTemplate {
    problems: AllProblems,
    download_path_nl: String,
    download_path_fry: String,
    frisian_export_allowed: bool,
}

pub async fn index(
    _: SubmitPath,
    context: Context,
    store: AppStore,
) -> Result<impl IntoResponse, AppError> {
    let problems = AllProblems::find_all(&store)?;

    Ok(HtmlTemplate(
        IndexTemplate {
            problems,
            download_path_nl: super::DownloadDocumentsPath {
                locale: ModelLocale::Nl,
            }
            .to_string(),
            download_path_fry: super::DownloadDocumentsPath {
                locale: ModelLocale::Fry,
            }
            .to_string(),
            frisian_export_allowed: context.election.frisian_export_allowed(),
        },
        context,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        AppStore, Context, ElectionConfig, ElectoralDistrict, Locale, Session,
        candidate_lists::CandidateListId,
        list_submitters::ListSubmitterId,
        persons::PersonId,
        test_utils::{
            response_body_string, sample_candidate_list, sample_list_submitter, sample_person,
        },
    };
    use axum::response::IntoResponse;

    #[tokio::test]
    async fn index_shows_document_downloads_for_complete_lists() -> Result<(), AppError> {
        let store = AppStore::new_for_test();
        let complete_list_id = CandidateListId::new();
        let person_id = PersonId::new();

        sample_list_submitter(ListSubmitterId::new())
            .update(&store)
            .await?;
        sample_person(person_id).create(&store).await?;

        let mut complete_list = sample_candidate_list(complete_list_id);
        complete_list.create(&store).await?;
        complete_list.append_candidate(&store, person_id).await?;

        let response = index(SubmitPath, Context::new_test_without_db(), store)
            .await?
            .into_response();
        let body = response_body_string(response).await;

        assert!(
            body.contains(
                &super::super::DownloadDocumentsPath {
                    locale: ModelLocale::Nl,
                }
                .to_string()
            )
        );

        assert!(
            body.matches(
                &super::super::DownloadDocumentsPath {
                    locale: ModelLocale::Nl,
                }
                .to_string()
            )
            .count()
                == 1
        );

        Ok(())
    }

    #[tokio::test]
    async fn index_shows_nl_and_fry_downloads_when_needed() -> Result<(), AppError> {
        for (election, district) in [
            (
                ElectionConfig::PS27(crate::Province::FR),
                ElectoralDistrict::FR,
            ),
            (
                ElectionConfig::WS27(crate::WaterCouncil::Fryslan),
                ElectoralDistrict::WsFryslan,
            ),
        ] {
            let store = AppStore::new_for_test_with_election(election);
            let complete_list_id = CandidateListId::new();
            let person_id = PersonId::new();

            sample_list_submitter(ListSubmitterId::new())
                .update(&store)
                .await?;
            sample_person(person_id).create(&store).await?;

            let mut complete_list = sample_candidate_list(complete_list_id);
            complete_list.electoral_districts = vec![district];
            complete_list.create(&store).await?;
            complete_list.append_candidate(&store, person_id).await?;

            let response = index(
                SubmitPath,
                Context::new(&store, Session::new_test_with_locale(Locale::Nl)),
                store,
            )
            .await?
            .into_response();
            let body = response_body_string(response).await;

            assert!(
                body.contains(
                    &super::super::DownloadDocumentsPath {
                        locale: ModelLocale::Nl,
                    }
                    .to_string()
                )
            );

            assert!(
                body.contains(
                    &super::super::DownloadDocumentsPath {
                        locale: ModelLocale::Fry,
                    }
                    .to_string()
                ),
                "Expected Frisian link for {:?}\n{}",
                election,
                body
            );
        }

        Ok(())
    }

    #[tokio::test]
    async fn index_shows_only_nl_when_needed() -> Result<(), AppError> {
        for (election, district) in [
            (ElectionConfig::EK27, ElectoralDistrict::FR),
            (
                ElectionConfig::PS27(crate::Province::GR),
                ElectoralDistrict::GR,
            ),
            (
                ElectionConfig::WS27(crate::WaterCouncil::Noorderzijlvest),
                ElectoralDistrict::WsNoorderzijlvest,
            ),
        ] {
            let store = AppStore::new_for_test_with_election(election);
            let complete_list_id = CandidateListId::new();
            let person_id = PersonId::new();

            sample_list_submitter(ListSubmitterId::new())
                .update(&store)
                .await?;
            sample_person(person_id).create(&store).await?;

            let mut complete_list = sample_candidate_list(complete_list_id);
            complete_list.electoral_districts = vec![district];
            complete_list.create(&store).await?;
            complete_list.append_candidate(&store, person_id).await?;

            let response = index(
                SubmitPath,
                Context::new(&store, Session::new_test_with_locale(Locale::Nl)),
                store,
            )
            .await?
            .into_response();
            let body = response_body_string(response).await;

            assert!(
                body.contains(
                    &super::super::DownloadDocumentsPath {
                        locale: ModelLocale::Nl,
                    }
                    .to_string()
                )
            );

            assert!(
                !body.contains(
                    &super::super::DownloadDocumentsPath {
                        locale: ModelLocale::Fry,
                    }
                    .to_string()
                ),
                "Expected no Frisian link for {:?}\n{}",
                election,
                body
            );
        }

        Ok(())
    }
}
