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

    Ok(candidates
        .iter()
        .map(|c| Candidate::try_from(&c.person, locale))
        .collect::<Result<Vec<_>, _>>()?)
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
                .ok_or(AppError::IncompleteData(
                    "Missing list submitter address",
                ))?,
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
                .ok_or(AppError::IncompleteData(
                    "Missing list submitter locality",
                ))?
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
    use crate::submit::structs::h1::ElectoralDistricts;

    #[test]
    fn test() {
        println!(
            "{}",
            serde_json::to_string_pretty(&ElectoralDistricts::All).unwrap()
        );

        println!(
            "{}",
            serde_json::to_string_pretty(&ElectoralDistricts::Some(vec!["asd".to_string()]))
                .unwrap()
        );
    }
}
