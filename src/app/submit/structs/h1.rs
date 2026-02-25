use crate::{
    AppError, ElectionConfig, Locale, Store,
    candidate_lists::{CandidateList, FullCandidateList},
    common::{Initials, PostalCode},
    core::{ElectionType, Pdf},
    list_submitters::ListSubmitter,
    persons::Person,
    substitute_list_submitters::SubstituteSubmitter,
};
use chrono::{Datelike, Timelike, Utc};
use serde::Serialize;
use tracing::error;

#[derive(Debug, Serialize)]
pub struct H1 {
    election_name: String,
    election_type: ElectionType,
    electoral_districts: ElectoralDistricts,
    designation: String,
    candidates: Vec<Candidate>,
    previously_seated: bool,
    list_submitter: BasicTypstPerson,
    substitute_submitter: Vec<BasicTypstPerson>,
    timestamp: Timestamp,
}

#[derive(Debug, Serialize)]
struct Timestamp {
    year: i32,
    month: u32,
    day: u32,
    hour: u32,
    minute: u32,
    second: u32,
}

#[derive(Debug, Serialize)]
struct Date {
    year: i32,
    month: u32,
    day: u32,
}

impl From<crate::common::Date> for Date {
    fn from(date: crate::common::Date) -> Self {
        Self {
            year: date.year(),
            month: date.month(),
            day: date.day(),
        }
    }
}

impl Timestamp {
    fn now() -> Self {
        let now = Utc::now();
        Self {
            year: now.year(),
            month: now.month(),
            day: now.day(),
            hour: now.hour(),
            minute: now.minute(),
            second: now.second(),
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(tag = "tag", content = "districts")]
enum ElectoralDistricts {
    All,
    Some(Vec<String>),
}

impl ElectoralDistricts {
    fn from(list: &CandidateList, election_config: &ElectionConfig) -> Self {
        if list.contains_all_districts(election_config) {
            ElectoralDistricts::All
        } else {
            ElectoralDistricts::Some(
                list.electoral_districts
                    .iter()
                    .map(|d| d.title().to_string())
                    .collect(),
            )
        }
    }
}

impl Pdf for H1 {
    fn typst_template_name(&self) -> &'static str {
        "model-h-1.typ"
    }

    fn filename(&self) -> &'static str {
        "h1.pdf"
    }
}

impl H1 {
    pub fn new(
        store: &Store,
        FullCandidateList {
            list,
            mut candidates,
        }: FullCandidateList,
        election: &ElectionConfig,
        locale: Locale,
    ) -> Result<Self, AppError> {
        Ok(Self {
            election_name: election.title().to_string(),
            election_type: election.election_type(),
            electoral_districts: ElectoralDistricts::from(&list, election),
            designation: store
                .get_political_group()?
                .display_name
                .ok_or(AppError::IncompleteData(
                    "Missing registered designation from political group",
                ))?
                .to_string(),
            candidates: ordered_candidates(&mut candidates, locale)?,
            // TODO
            previously_seated: true,
            list_submitter: store
                .get_list_submitter(
                    list.list_submitter_id
                        .ok_or(AppError::IncompleteData("Missing list submitter"))?,
                )?
                .try_into()?,
            substitute_submitter: substitute_submitter_from_ids(&list, store.clone())?,
            timestamp: Timestamp::now(),
        })
    }
}

#[derive(Debug, Serialize)]
struct Candidate {
    last_name: String,
    /// Initials as printed on the model, e.g., optionally including the gender and first name
    initials: String,
    date_of_birth: Date,
    locality: String,
}

impl Candidate {
    fn try_from(person: &Person, locale: Locale) -> Result<Self, AppError> {
        Ok(Self {
            last_name: person.display_name(),
            initials: person.initials_as_printed_on_list(locale),
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
    locale: Locale,
) -> Result<Vec<Candidate>, AppError> {
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
        .map(|c| Candidate::try_from(&c.person, locale))
        .collect::<Result<Vec<_>, _>>()
}

#[derive(Debug, Serialize)]
struct BasicTypstPerson {
    last_name: String,
    initials: Initials,
    postal_address: String,
    postal_code: PostalCode,
    locality: String,
}

impl TryFrom<SubstituteSubmitter> for BasicTypstPerson {
    type Error = AppError;

    fn try_from(submitter: SubstituteSubmitter) -> Result<Self, Self::Error> {
        Ok(BasicTypstPerson {
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

impl TryFrom<ListSubmitter> for BasicTypstPerson {
    type Error = AppError;

    fn try_from(submitter: ListSubmitter) -> Result<Self, Self::Error> {
        Ok(BasicTypstPerson {
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
    store: Store,
) -> Result<Vec<BasicTypstPerson>, AppError> {
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
        AppError, ElectionConfig, ElectoralDistrict, Locale, Store,
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
        let date = Date::from(input);

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
            ElectoralDistricts::from(&list, &election),
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

        match ElectoralDistricts::from(&list, &election) {
            ElectoralDistricts::Some(districts) => {
                assert_eq!(
                    districts,
                    vec!["Utrecht".to_string(), "Noord-Holland".to_string()]
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

        let ordered = ordered_candidates(&mut candidates, Locale::Nl)?;

        assert_eq!(ordered.len(), 2);
        assert_eq!(ordered[0].last_name, "Henk Beta");
        assert_eq!(ordered[1].last_name, "Henk Alpha");
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

        let err = ordered_candidates(&mut candidates, Locale::Nl).unwrap_err();
        assert!(matches!(err, AppError::IntegrityViolation));
    }

    #[test]
    fn candidate_requires_birth_date() {
        let mut person = sample_person(PersonId::new());
        person.date_of_birth = None;

        let err = Candidate::try_from(&person, Locale::Nl).unwrap_err();
        assert!(matches!(
            err,
            AppError::IncompleteData("Missing birth date for candidate")
        ));
    }

    #[test]
    fn candidate_requires_locality() {
        let mut person = sample_person(PersonId::new());
        person.place_of_residence = None;

        let err = Candidate::try_from(&person, Locale::Nl).unwrap_err();
        assert!(matches!(
            err,
            AppError::IncompleteData("Missing locality for candidate")
        ));
    }

    #[test]
    fn basic_typst_person_from_list_submitter_maps_fields() -> Result<(), AppError> {
        let submitter = sample_list_submitter(ListSubmitterId::new());
        let basic = BasicTypstPerson::try_from(submitter)?;

        assert_eq!(basic.last_name, "Bos");
        assert_eq!(
            basic.initials,
            "E.F.".parse::<Initials>().expect("initials")
        );
        assert_eq!(basic.postal_address, "Coolsingel 5B");
        assert_eq!(
            basic.postal_code,
            "3011 CC".parse::<PostalCode>().expect("postal code")
        );
        assert_eq!(basic.locality, "Rotterdam");

        Ok(())
    }

    #[test]
    fn basic_typst_person_from_list_submitter_requires_postal_code() {
        let mut submitter = sample_list_submitter(ListSubmitterId::new());
        submitter.address.postal_code = None;

        let err = BasicTypstPerson::try_from(submitter).unwrap_err();
        assert!(matches!(
            err,
            AppError::IncompleteData("Missing list submitter postal code")
        ));
    }

    #[test]
    fn basic_typst_person_from_substitute_submitter_requires_address_line() {
        let mut submitter = sample_substitute_submitter(SubstituteSubmitterId::new());
        submitter.address.street_name = None;

        let err = BasicTypstPerson::try_from(submitter).unwrap_err();
        assert!(matches!(
            err,
            AppError::IncompleteData("Missing substitute submitter address")
        ));
    }

    #[tokio::test]
    async fn substitute_submitter_from_ids_resolves_submitters() -> Result<(), AppError> {
        let store = Store::new_for_test().await;
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
        let store = Store::new_for_test().await;
        let list = CandidateList {
            substitute_list_submitter_ids: vec![SubstituteSubmitterId::new()],
            ..Default::default()
        };

        let err = substitute_submitter_from_ids(&list, store).unwrap_err();
        assert!(matches!(err, AppError::IntegrityViolation));
    }
}
