use crate::{
    AppError, AppStore, ElectionConfig,
    candidate_lists::{CandidateList, FullCandidateList},
    common::{Initials, PostalCode},
    core::{ElectionType, ModelLocale, Pdf},
    list_submitters::ListSubmitter,
    persons::Person,
    submit::structs::{TypstDate, TypstDatetime},
    substitute_list_submitters::SubstituteSubmitter,
};
use serde::Serialize;
use tracing::error;

#[derive(Debug, Serialize)]
pub struct H1 {
    election_name: String,
    election_type: ElectionType,
    electoral_districts: ElectoralDistricts,
    designation: String,
    candidates: Vec<TypstCandidate>,
    previously_seated: bool,
    list_submitter: TypstPerson,
    substitute_submitter: Vec<TypstPerson>,
    timestamp: TypstDatetime,
    locale: ModelLocale,
}

#[derive(Debug, Serialize)]
#[serde(tag = "tag", content = "districts")]
enum ElectoralDistricts {
    All,
    Some(Vec<String>),
}

impl ElectoralDistricts {
    fn from(list: &CandidateList, election_config: &ElectionConfig, locale: ModelLocale) -> Self {
        if list.contains_all_districts(election_config) {
            ElectoralDistricts::All
        } else {
            ElectoralDistricts::Some(
                list.electoral_districts
                    .iter()
                    .map(|d| d.title(locale.into()).to_string())
                    .collect(),
            )
        }
    }
}

impl Pdf for H1 {
    fn typst_template_name(&self) -> String {
        format!("model-h1-{}.typ", self.locale)
    }

    fn filename(&self) -> String {
        format!("model-h1-{}.pdf", self.locale)
    }
}

impl H1 {
    pub fn new(
        store: &AppStore,
        FullCandidateList {
            list,
            mut candidates,
        }: FullCandidateList,
        election: &ElectionConfig,
        locale: ModelLocale,
    ) -> Result<Self, AppError> {
        Ok(Self {
            election_name: election.title(locale.into()).to_string(),
            election_type: election.election_type(),
            electoral_districts: ElectoralDistricts::from(&list, election, locale),
            designation: store
                .get_political_group()?
                .display_name
                .ok_or(AppError::IncompleteData(
                    "Missing registered designation from political group",
                ))?
                .to_string(),
            candidates: ordered_candidates(&mut candidates, locale)?,
            previously_seated: true,
            list_submitter: store
                .get_list_submitter(
                    list.list_submitter_id
                        .ok_or(AppError::IncompleteData("Missing list submitter"))?,
                )?
                .try_into()?,
            substitute_submitter: substitute_submitter_from_ids(&list, store.clone())?,
            timestamp: TypstDatetime::now(),
            locale,
        })
    }
}

#[derive(Debug, Serialize)]
struct TypstCandidate {
    last_name: String,
    /// Initials as printed on the model, e.g., optionally including the gender and first name
    initials: String,
    date_of_birth: TypstDate,
    locality: String,
}

impl TypstCandidate {
    fn try_from(person: &Person, locale: ModelLocale) -> Result<Self, AppError> {
        Ok(Self {
            last_name: person.name.last_name_with_prefix(),
            initials: person.initials_as_printed_on_list(locale.into()),
            date_of_birth: person
                .date_of_birth
                .ok_or(AppError::IncompleteData("Missing birth date for candidate"))?
                .into(),
            locality: person
                .place_of_residence
                .clone()
                .ok_or(AppError::IncompleteData("Missing locality for candidate"))?
                .to_string(),
        })
    }
}

fn ordered_candidates(
    candidates: &mut [crate::candidates::Candidate],
    locale: ModelLocale,
) -> Result<Vec<TypstCandidate>, AppError> {
    candidates.sort_by(|a, b| a.position.cmp(&b.position));
    for (i, candidate) in candidates.iter().enumerate() {
        if candidate.position != i + 1 {
            error!(
                "Found a hole in candidate list: expected position {}, got {} for candidate {}",
                i + 1,
                candidate.position,
                candidate.person.display_name()
            );
            return Err(AppError::IntegrityViolation);
        }
    }

    candidates
        .iter()
        .map(|c| TypstCandidate::try_from(&c.person, locale))
        .collect::<Result<Vec<_>, _>>()
}

#[derive(Debug, Serialize)]
struct TypstPerson {
    last_name: String,
    initials: Initials,
    postal_address: String,
    postal_code: PostalCode,
    locality: String,
}

impl TryFrom<SubstituteSubmitter> for TypstPerson {
    type Error = AppError;

    fn try_from(submitter: SubstituteSubmitter) -> Result<Self, Self::Error> {
        Ok(TypstPerson {
            last_name: submitter.name.last_name.to_string(),
            initials: submitter.name.initials,
            postal_address: submitter
                .address
                .address_line_1()
                .ok_or(AppError::IncompleteData(
                    "Missing substitute submitter address",
                ))?,
            postal_code: submitter
                .address
                .postal_code
                .clone()
                .ok_or(AppError::IncompleteData(
                    "Missing substitute submitter postal code",
                ))?,
            locality: submitter
                .address
                .locality
                .clone()
                .ok_or(AppError::IncompleteData(
                    "Missing substitute submitter locality",
                ))?
                .to_string(),
        })
    }
}

impl TryFrom<ListSubmitter> for TypstPerson {
    type Error = AppError;

    fn try_from(submitter: ListSubmitter) -> Result<Self, Self::Error> {
        Ok(TypstPerson {
            last_name: submitter.name.last_name.to_string(),
            initials: submitter.name.initials,
            postal_address: submitter
                .address
                .address_line_1()
                .ok_or(AppError::IncompleteData("Missing list submitter address"))?,
            postal_code: submitter
                .address
                .postal_code
                .clone()
                .ok_or(AppError::IncompleteData(
                    "Missing list submitter postal code",
                ))?,
            locality: submitter
                .address
                .locality
                .clone()
                .ok_or(AppError::IncompleteData("Missing list submitter locality"))?
                .to_string(),
        })
    }
}

fn substitute_submitter_from_ids(
    list: &CandidateList,
    store: AppStore,
) -> Result<Vec<TypstPerson>, AppError> {
    list.substitute_list_submitter_ids
        .iter()
        .map(|id| match store.get_substitute_submitter(*id) {
            Ok(submitter) => submitter.try_into(),
            Err(_) => Err(AppError::IntegrityViolation),
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        AppError, AppStore, ElectionConfig, ElectoralDistrict,
        candidate_lists::{CandidateList, CandidateListId},
        candidates::Candidate as AppCandidate,
        common::{Initials, LastName, PostalCode},
        list_submitters::ListSubmitterId,
        persons::PersonId,
        substitute_list_submitters::SubstituteSubmitterId,
        test_utils::{
            sample_list_submitter, sample_person, sample_person_with_last_name,
            sample_substitute_submitter,
        },
    };

    #[test]
    fn date_from_common_date_copies_components() {
        let input: crate::common::Date = "15-03-2001".parse().expect("date");
        let date = TypstDate::from(input);

        assert_eq!(date.year, 2001);
        assert_eq!(date.month, 3);
        assert_eq!(date.day, 15);
    }

    #[test]
    fn electoral_districts_from_full_list_returns_all() {
        let election = ElectionConfig::EK2027;
        let list = CandidateList {
            electoral_districts: election.electoral_districts().to_vec(),
            ..Default::default()
        };

        assert!(matches!(
            ElectoralDistricts::from(&list, &election, ModelLocale::Fry),
            ElectoralDistricts::All
        ));
    }

    #[test]
    fn electoral_districts_from_partial_list_returns_titles() {
        let election = ElectionConfig::EK2027;
        let list = CandidateList {
            electoral_districts: vec![ElectoralDistrict::UT, ElectoralDistrict::NH],
            ..Default::default()
        };

        match ElectoralDistricts::from(&list, &election, ModelLocale::Nl) {
            ElectoralDistricts::Some(districts) => {
                assert_eq!(
                    districts,
                    vec!["Utrecht".to_string(), "Noord-Holland".to_string()]
                );
            }
            ElectoralDistricts::All => panic!("expected Some districts"),
        }
        match ElectoralDistricts::from(&list, &election, ModelLocale::Fry) {
            ElectoralDistricts::Some(districts) => {
                assert_eq!(
                    districts,
                    vec!["Utert".to_string(), "Noard-Hollân".to_string()]
                );
            }
            ElectoralDistricts::All => panic!("expected Some districts"),
        }
    }

    #[test]
    fn ordered_candidates_sorts_and_maps_people() -> Result<(), AppError> {
        let list_id = CandidateListId::new();
        let person_a = sample_person_with_last_name(PersonId::new(), "Alpha");
        let person_b = sample_person_with_last_name(PersonId::new(), "Beta");

        let mut candidates = vec![
            AppCandidate {
                list_id,
                position: 2,
                person: person_a,
            },
            AppCandidate {
                list_id,
                position: 1,
                person: person_b,
            },
        ];

        let ordered = ordered_candidates(&mut candidates, ModelLocale::Nl)?;

        assert_eq!(ordered.len(), 2);
        assert_eq!(ordered[0].last_name, "Beta");
        assert_eq!(ordered[1].last_name, "Alpha");
        assert_eq!(ordered[0].date_of_birth.year, 1990);
        assert_eq!(ordered[0].date_of_birth.month, 2);
        assert_eq!(ordered[0].date_of_birth.day, 1);
        assert_eq!(ordered[0].locality, "Juinen");

        Ok(())
    }

    #[test]
    fn ordered_candidates_returns_error_on_hole() {
        let list_id = CandidateListId::new();
        let mut candidates = vec![
            AppCandidate {
                list_id,
                position: 1,
                person: sample_person(PersonId::new()),
            },
            AppCandidate {
                list_id,
                position: 3,
                person: sample_person(PersonId::new()),
            },
        ];

        let err = ordered_candidates(&mut candidates, ModelLocale::Nl).unwrap_err();
        assert!(matches!(err, AppError::IntegrityViolation));
    }

    #[test]
    fn candidate_requires_birth_date() {
        let mut person = sample_person(PersonId::new());
        person.date_of_birth = None;

        let err = TypstCandidate::try_from(&person, ModelLocale::Nl).unwrap_err();
        assert!(matches!(
            err,
            AppError::IncompleteData("Missing birth date for candidate")
        ));
    }

    #[test]
    fn candidate_requires_locality() {
        let mut person = sample_person(PersonId::new());
        person.place_of_residence = None;

        let err = TypstCandidate::try_from(&person, ModelLocale::Nl).unwrap_err();
        assert!(matches!(
            err,
            AppError::IncompleteData("Missing locality for candidate")
        ));
    }

    #[test]
    fn typst_person_from_list_submitter_maps_fields() -> Result<(), AppError> {
        let submitter = sample_list_submitter(ListSubmitterId::new());
        let person = TypstPerson::try_from(submitter)?;

        assert_eq!(person.last_name, "Bos");
        assert_eq!(
            person.initials,
            "E.F.".parse::<Initials>().expect("initials")
        );
        assert_eq!(person.postal_address, "Coolsingel 5B");
        assert_eq!(
            person.postal_code,
            "3011 CC".parse::<PostalCode>().expect("postal code")
        );
        assert_eq!(person.locality, "Rotterdam");

        Ok(())
    }

    #[test]
    fn typst_person_from_list_submitter_requires_postal_code() {
        let mut submitter = sample_list_submitter(ListSubmitterId::new());
        submitter.address.postal_code = None;

        let err = TypstPerson::try_from(submitter).unwrap_err();
        assert!(matches!(
            err,
            AppError::IncompleteData("Missing list submitter postal code")
        ));
    }

    #[test]
    fn typst_person_from_substitute_submitter_requires_address_line() {
        let mut submitter = sample_substitute_submitter(SubstituteSubmitterId::new());
        submitter.address.street_name = None;

        let err = TypstPerson::try_from(submitter).unwrap_err();
        assert!(matches!(
            err,
            AppError::IncompleteData("Missing substitute submitter address")
        ));
    }

    #[tokio::test]
    async fn substitute_submitter_from_ids_resolves_submitters() -> Result<(), AppError> {
        let store = AppStore::new_for_test().await;
        let submitter_a = sample_substitute_submitter(SubstituteSubmitterId::new());
        let mut submitter_b = sample_substitute_submitter(SubstituteSubmitterId::new());
        submitter_b.name.last_name = "Janssen".parse::<LastName>().expect("last name");

        submitter_a.create(&store).await?;
        submitter_b.create(&store).await?;

        let list = CandidateList {
            substitute_list_submitter_ids: vec![submitter_a.id, submitter_b.id],
            ..Default::default()
        };

        let resolved = substitute_submitter_from_ids(&list, store)?;

        assert_eq!(resolved.len(), 2);
        assert_eq!(resolved[0].last_name, "Bakker");
        assert_eq!(resolved[1].last_name, "Janssen");

        Ok(())
    }

    #[tokio::test]
    async fn substitute_submitter_from_ids_returns_integrity_error_on_missing() {
        let store = AppStore::new_for_test().await;
        let list = CandidateList {
            substitute_list_submitter_ids: vec![SubstituteSubmitterId::new()],
            ..Default::default()
        };

        let err = substitute_submitter_from_ids(&list, store).unwrap_err();
        assert!(matches!(err, AppError::IntegrityViolation));
    }
}
