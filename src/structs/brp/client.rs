use std::{str::FromStr, time::Duration};

use chrono::NaiveDate;
use reqwest::Client;
use secrecy::{ExposeSecret, SecretString};
use serde::{Deserialize, Serialize};

use super::{BrpCheckedField, BrpField, BrpFinding, BrpPerson, person::BrpResidence};
use crate::{
    AppError, ElectionConfig,
    constants::DEFAULT_DATE_FORMAT,
    structs::{
        common::{Bsn, BsnOrNoneConfirmed, DateOfBirth, LastName, LastNamePrefix},
        persons::{Person, PersonId},
    },
};

/// The BRP accepts at most twenty burgerservicenummers in one
/// `RaadpleegMetBurgerservicenummer` request; candidates are looked up ten at
/// a time, well inside that limit.
pub const BRP_BSN_BATCH_SIZE: usize = 10;

/// The fields requested per candidate: everything printed on the candidate
/// list (model H 1), the date of birth and burgerservicenummer, and what is
/// needed to establish eligibility.
pub const CANDIDATE_FIELDS: &[BrpField] = &[
    BrpField::Bsn,
    BrpField::Initials,
    BrpField::LastNamePrefix,
    BrpField::LastName,
    BrpField::Gender,
    BrpField::DateOfBirth,
    BrpField::PlaceOfResidence,
    BrpField::DateOfDeath,
    BrpField::Nationality,
    BrpField::SuffrageExclusion,
];

/// The `geslacht` code the BRP uses when the gender is unknown.
const GENDER_UNKNOWN_CODE: &str = "O";

/// The date format the BRP uses for a complete date.
const BRP_DATE_FORMAT: &str = "%Y-%m-%d";

#[derive(Clone)]
pub struct BrpClient {
    http_client: Client,
    base_url: String,
    api_key: SecretString,
    persons_endpoint: String,
    timeout: Duration,
}

impl BrpClient {
    pub fn new(
        base_url: &str,
        api_key: SecretString,
        persons_endpoint: &str,
        timeout: Duration,
    ) -> Self {
        Self {
            http_client: Client::new(),
            base_url: base_url.to_string(),
            api_key,
            persons_endpoint: persons_endpoint.to_string(),
            timeout,
        }
    }

    /// A client pointed at `base_url`, for tests that serve their own BRP
    /// responses instead of reaching for the mock container.
    #[cfg(test)]
    pub fn new_for_test(base_url: &str) -> Self {
        use crate::constants;

        BrpClient::new(
            base_url,
            SecretString::from(""),
            constants::BRP_PERSONS_ENDPOINT,
            Duration::from_secs(5),
        )
    }

    pub async fn get_persons(&self, query: &BrpQuery) -> Result<Vec<BrpPerson>, AppError> {
        let url = format!("{}/{}", self.base_url, self.persons_endpoint);

        let response = self
            .http_client
            .post(&url)
            .header(
                "Authorization",
                format!("Bearer {}", self.api_key.expose_secret()),
            )
            .json(query)
            .timeout(self.timeout)
            .send()
            .await?
            .error_for_status()?;

        match response.json::<BrpResponse>().await? {
            BrpResponse::ConsultWithBsn { persons } => Ok(persons),
        }
    }

    /// Check up to [`BRP_BSN_BATCH_SIZE`] candidates in a single BRP request
    /// and return the findings per candidate.
    ///
    /// An `Err` means the BRP could not be consulted at all, and none of these
    /// candidates were checked; the caller is expected to stop rather than to
    /// treat the batch as clean. Everything that is wrong with an individual
    /// candidate is a [`BrpFinding`], not an error.
    pub async fn verify_batch(
        &self,
        persons: &[Person],
        election: &ElectionConfig,
    ) -> Result<Vec<(PersonId, Vec<BrpFinding>)>, AppError> {
        if persons.len() > BRP_BSN_BATCH_SIZE {
            return Err(AppError::BrpError(format!(
                "a batch of {} candidates exceeds the batch size of {BRP_BSN_BATCH_SIZE}",
                persons.len()
            )));
        }

        // A candidate without a burgerservicenummer cannot be looked up. That
        // is a finding about the candidate, not a failure of the check, so it
        // must not abort the sweep.
        let (with_bsn, without_bsn): (Vec<&Person>, Vec<&Person>) =
            persons.iter().partition(|person| bsn_of(person).is_some());

        let mut results: Vec<(PersonId, Vec<BrpFinding>)> = without_bsn
            .into_iter()
            .map(|person| {
                let finding = match person.personal_data.bsn {
                    Some(BsnOrNoneConfirmed::NoneConfirmed) => BrpFinding::BsnNoneConfirmed,
                    _ => BrpFinding::BsnMissing,
                };
                (person.id, vec![finding])
            })
            .collect();

        if with_bsn.is_empty() {
            return Ok(results);
        }

        let query = BrpQuery::ConsultWithBsn {
            bsn: with_bsn.iter().filter_map(|p| bsn_of(p).cloned()).collect(),
            fields: CANDIDATE_FIELDS.to_vec(),
        };
        let brp_persons = self.get_persons(&query).await?;

        for person in with_bsn {
            let Some(bsn) = bsn_of(person) else {
                continue;
            };

            // Responses come back as one list, so each candidate is matched to
            // their own record by burgerservicenummer.
            let matched: Vec<&BrpPerson> = brp_persons
                .iter()
                .filter(|brp_person| brp_person.bsn.as_deref() == Some(bsn.expose()))
                .collect();

            let findings = match matched.as_slice() {
                [] => vec![BrpFinding::BsnUnknown],
                [brp_person] => findings_for(person, brp_person, election),
                [..] => vec![BrpFinding::BsnNotUnique],
            };

            results.push((person.id, findings));
        }

        Ok(results)
    }
}

/// The candidate's burgerservicenummer, if they have one recorded.
fn bsn_of(person: &Person) -> Option<&Bsn> {
    match &person.personal_data.bsn {
        Some(BsnOrNoneConfirmed::Bsn(bsn)) => Some(bsn),
        _ => None,
    }
}

/// Everything the BRP says about one candidate that the committee should see.
fn findings_for(
    person: &Person,
    brp_person: &BrpPerson,
    election: &ElectionConfig,
) -> Vec<BrpFinding> {
    let mut findings = Vec::new();

    findings.extend(name_finding(person, brp_person));
    findings.extend(initials_finding(person, brp_person));
    findings.extend(gender_finding(person, brp_person));
    findings.extend(date_of_birth_finding(person, brp_person));
    findings.extend(place_of_residence_finding(person, brp_person));
    findings.extend(eligibility_findings(brp_person, election));

    findings
}

/// Compare one candidate value with the BRP's, keeping the three outcomes
/// apart: the BRP holds no value, the BRP holds a value this application
/// cannot interpret, or the two differ.
fn compare<T>(field: BrpCheckedField, ours: Option<&T>, theirs: Option<&str>) -> Option<BrpFinding>
where
    T: FromStr + PartialEq,
{
    // A field the BRP left out is as unverified as one it returned empty; both
    // have to say so rather than pass silently.
    let raw = theirs.map(str::trim).unwrap_or_default();
    if raw.is_empty() {
        return Some(BrpFinding::MissingInBrp { field });
    }

    let Ok(parsed) = raw.parse::<T>() else {
        return Some(BrpFinding::Unparsable {
            field,
            brp_value: raw.to_string(),
        });
    };

    match ours {
        Some(ours) if *ours == parsed => None,
        _ => Some(BrpFinding::Mismatch {
            field,
            brp_value: raw.to_string(),
        }),
    }
}

/// The last name and its prefix are reported as one finding, because the
/// candidate detail table shows them on one row.
fn name_finding(person: &Person, brp_person: &BrpPerson) -> Option<BrpFinding> {
    let field = BrpCheckedField::LastName;
    let name = brp_person.name.as_ref();

    let Some(last_name) = name
        .and_then(|name| name.last_name.as_deref())
        .map(str::trim)
        .filter(|last_name| !last_name.is_empty())
    else {
        return Some(BrpFinding::MissingInBrp { field });
    };

    let prefix = name
        .and_then(|name| name.last_name_prefix.as_deref())
        .map(str::trim)
        .filter(|prefix| !prefix.is_empty());

    // The BRP's own spelling is what the committee needs to see, so the raw
    // values are combined for display rather than the parsed ones.
    let brp_value = match prefix {
        Some(prefix) => format!("{prefix} {last_name}"),
        None => last_name.to_string(),
    };

    // A last name or prefix this application cannot parse leaves the name
    // uncomparable, which is not the same as the two names differing.
    let (Ok(brp_last_name), Ok(brp_prefix)) = (
        last_name.parse::<LastName>(),
        prefix.map(str::parse::<LastNamePrefix>).transpose(),
    ) else {
        return Some(BrpFinding::Unparsable { field, brp_value });
    };

    let matches =
        person.name.last_name == brp_last_name && person.name.last_name_prefix == brp_prefix;

    (!matches).then_some(BrpFinding::Mismatch { field, brp_value })
}

fn initials_finding(person: &Person, brp_person: &BrpPerson) -> Option<BrpFinding> {
    compare(
        BrpCheckedField::Initials,
        Some(&person.name.initials),
        brp_person
            .name
            .as_ref()
            .and_then(|name| name.initials.as_deref()),
    )
}

/// The gender is only printed on the list when the candidate supplied one, so
/// it is only compared when they did.
fn gender_finding(person: &Person, brp_person: &BrpPerson) -> Option<BrpFinding> {
    let ours = person.personal_data.gender.as_ref()?;
    let field = BrpCheckedField::Gender;

    // "O" is the BRP's own "unknown", not a value this application failed to
    // parse.
    match brp_person.gender_code() {
        Some(code) if code.eq_ignore_ascii_case(GENDER_UNKNOWN_CODE) => {
            Some(BrpFinding::MissingInBrp { field })
        }
        code => compare(field, Some(ours), code),
    }
}

fn date_of_birth_finding(person: &Person, brp_person: &BrpPerson) -> Option<BrpFinding> {
    let field = BrpCheckedField::DateOfBirth;
    let Some(raw) = brp_person.date_of_birth() else {
        // Reported as `AgeUnknown` by the eligibility check instead; a missing
        // date of birth is an eligibility problem, not a difference.
        return None;
    };

    match parse_brp_date(raw) {
        None => Some(BrpFinding::Unparsable {
            field,
            brp_value: raw.to_string(),
        }),
        Some(date) if person.personal_data.date_of_birth.as_ref() == Some(&date) => None,
        Some(date) => Some(BrpFinding::Mismatch {
            field,
            brp_value: format_date(&date),
        }),
    }
}

/// The `woonplaats` is the one address element printed on the candidate list.
/// The three residence shapes that carry no `woonplaats` each get their own
/// finding, so "lives abroad" is never reported as "residence unknown".
fn place_of_residence_finding(person: &Person, brp_person: &BrpPerson) -> Option<BrpFinding> {
    let field = BrpCheckedField::PlaceOfResidence;

    match &brp_person.residence {
        Some(BrpResidence::Address { address }) => compare(
            field,
            person.personal_data.place_of_residence.as_ref(),
            address
                .as_ref()
                .and_then(|address| address.place_of_residence.as_deref()),
        ),
        Some(BrpResidence::Abroad) => Some(BrpFinding::ResidenceAbroad),
        Some(BrpResidence::Location) => Some(BrpFinding::ResidenceWithoutAddress),
        Some(BrpResidence::Unknown | BrpResidence::Other) | None => {
            Some(BrpFinding::ResidenceUnknown)
        }
    }
}

/// Whether the BRP shows this candidate can be elected at all: article 56 of
/// the Grondwet requires a Dutch national who has reached the age of eighteen
/// and is not excluded from the right to vote -- and, self-evidently, someone
/// who is still alive.
fn eligibility_findings(brp_person: &BrpPerson, election: &ElectionConfig) -> Vec<BrpFinding> {
    let mut findings = Vec::new();

    if let Some(date_of_death) = brp_person.date_of_death() {
        findings.push(BrpFinding::Deceased {
            date_of_death: parse_brp_date(date_of_death)
                .map(|date| format_date(&date))
                .unwrap_or_else(|| date_of_death.to_string()),
        });
    }

    match brp_person.date_of_birth() {
        // No date at all: the age cannot be established.
        None => findings.push(BrpFinding::AgeUnknown),
        // A date that could not be read is already reported as `Unparsable` by
        // the field comparison, so it is not repeated here.
        Some(raw) => {
            if let Some(date_of_birth) = parse_brp_date(raw)
                && date_of_birth.is_too_young(election)
            {
                findings.push(BrpFinding::Underage {
                    date_of_birth: format_date(&date_of_birth),
                });
            }
        }
    }

    if !brp_person.is_dutch() {
        findings.push(BrpFinding::NotDutch);
    }

    if brp_person.is_excluded_from_suffrage() {
        findings.push(BrpFinding::ExcludedFromSuffrage);
    }

    findings
}

fn parse_brp_date(raw: &str) -> Option<DateOfBirth> {
    NaiveDate::parse_from_str(raw.trim(), BRP_DATE_FORMAT)
        .ok()
        .map(DateOfBirth::from)
}

/// A BRP date in the format the rest of the interface uses.
fn format_date(date: &DateOfBirth) -> String {
    date.format(DEFAULT_DATE_FORMAT).to_string()
}

#[derive(Debug, Serialize)]
#[serde(tag = "type")]
pub enum BrpQuery {
    #[serde(rename = "RaadpleegMetBurgerservicenummer")]
    ConsultWithBsn {
        #[serde(rename = "burgerservicenummer")]
        bsn: Vec<Bsn>,
        fields: Vec<BrpField>,
    },
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
pub enum BrpResponse {
    #[serde(rename = "RaadpleegMetBurgerservicenummer")]
    ConsultWithBsn {
        #[serde(rename = "personen")]
        persons: Vec<BrpPerson>,
    },
}

#[cfg(test)]
mod tests {
    use axum::{Router, routing::post};
    use serde_json::{Value, json};
    use tokio::net::TcpListener;

    use super::*;
    use crate::{
        brp_stub::{BrpStub, matching_record},
        constants,
        structs::{brp::BrpFinding, persons::PersonId},
        test_utils::{sample_person, sample_person_from_brp},
    };

    /// The findings for a single candidate whose record the stub serves.
    async fn findings_for_record(person: &Person, record: Value) -> Vec<BrpFinding> {
        let stub = BrpStub::serving(vec![record]).await;
        let results = stub
            .client
            .verify_batch(std::slice::from_ref(person), &ElectionConfig::EK27)
            .await
            .expect("the stub BRP answers");

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].0, person.id);
        results[0].1.clone()
    }

    /// `count` distinct burgerservicenummers, taken from the numbers that pass
    /// this application's own validation (length and the eleven-proof).
    fn valid_bsns(count: usize) -> Vec<Bsn> {
        let bsns: Vec<Bsn> = (100_000_000u32..)
            .filter_map(|number| number.to_string().parse::<Bsn>().ok())
            .take(count)
            .collect();
        assert_eq!(bsns.len(), count);
        bsns
    }

    /// The candidate fixture together with the BSN it is looked up by.
    fn candidate() -> (Person, String) {
        let person = sample_person_from_brp();
        let bsn = bsn_of(&person)
            .expect("the fixture has a BSN")
            .expose()
            .to_string();
        (person, bsn)
    }

    #[tokio::test]
    async fn a_matching_candidate_produces_no_findings() {
        let (person, bsn) = candidate();

        let findings = findings_for_record(&person, matching_record(&bsn)).await;

        assert!(
            findings.is_empty(),
            "expected no findings, got {findings:?}"
        );
    }

    #[tokio::test]
    async fn the_correspondence_address_is_not_checked() {
        let (mut person, bsn) = candidate();
        // Only the woonplaats is verified; the rest of the address is not
        // printed on the candidate list, so it must not produce a finding.
        person.address.street_name = Some("Heel Andere Laan".parse().unwrap());
        person.address.house_number = Some("999".parse().unwrap());
        person.address.house_number_addition = Some("Z".parse().unwrap());
        person.address.postal_code = Some("1234AB".parse().unwrap());
        person.address.locality = Some("Amsterdam".parse().unwrap());

        let findings = findings_for_record(&person, matching_record(&bsn)).await;

        assert!(
            findings.is_empty(),
            "expected no findings, got {findings:?}"
        );
    }

    #[tokio::test]
    async fn a_different_last_name_is_reported_once_including_its_prefix() {
        let (person, bsn) = candidate();
        let mut record = matching_record(&bsn);
        record["naam"]["geslachtsnaam"] = json!("Bruijn");

        let findings = findings_for_record(&person, record).await;

        assert_eq!(
            findings,
            vec![BrpFinding::Mismatch {
                field: BrpCheckedField::LastName,
                // The prefix travels with the name the committee is shown.
                brp_value: "de Bruijn".to_string(),
            }]
        );
    }

    #[tokio::test]
    async fn a_different_place_of_residence_is_reported() {
        let (person, bsn) = candidate();
        let mut record = matching_record(&bsn);
        record["verblijfplaats"]["verblijfadres"]["woonplaats"] = json!("Amsterdam");

        let findings = findings_for_record(&person, record).await;

        assert_eq!(
            findings,
            vec![BrpFinding::Mismatch {
                field: BrpCheckedField::PlaceOfResidence,
                brp_value: "Amsterdam".to_string(),
            }]
        );
    }

    #[tokio::test]
    async fn a_brp_value_that_cannot_be_parsed_is_not_reported_as_a_difference() {
        let (person, bsn) = candidate();
        let mut record = matching_record(&bsn);
        // The BRP allows initials this application's own type rejects. That is
        // a problem reading the BRP, not a statement that the party filled in
        // the wrong initials, and the two must not be conflated.
        record["naam"]["voorletters"] = json!("T4");

        let findings = findings_for_record(&person, record).await;

        assert_eq!(
            findings,
            vec![BrpFinding::Unparsable {
                field: BrpCheckedField::Initials,
                brp_value: "T4".to_string(),
            }]
        );
    }

    #[tokio::test]
    async fn an_unparsable_date_of_birth_is_reported_as_unparsable() {
        let (person, bsn) = candidate();
        let mut record = matching_record(&bsn);
        record["geboorte"]["datum"]["datum"] = json!("11-12-1990");

        let findings = findings_for_record(&person, record).await;

        // Reported once, as the field that could not be read -- not also as an
        // age that could not be established.
        assert_eq!(
            findings,
            vec![BrpFinding::Unparsable {
                field: BrpCheckedField::DateOfBirth,
                brp_value: "11-12-1990".to_string(),
            }]
        );
    }

    #[tokio::test]
    async fn a_gender_the_brp_does_not_know_is_missing_rather_than_unparsable() {
        let (person, bsn) = candidate();
        let mut record = matching_record(&bsn);
        record["geslacht"]["code"] = json!("O");

        let findings = findings_for_record(&person, record).await;

        assert_eq!(
            findings,
            vec![BrpFinding::MissingInBrp {
                field: BrpCheckedField::Gender
            }]
        );
    }

    #[tokio::test]
    async fn a_field_the_brp_left_out_is_reported_rather_than_passed_silently() {
        let (person, bsn) = candidate();
        let mut record = matching_record(&bsn);
        // The BRP returns a name without initials, so they could not be
        // verified. Saying nothing would read as "verified and correct".
        record["naam"] = json!({ "geslachtsnaam": "Bruin", "voorvoegsel": "de" });

        let findings = findings_for_record(&person, record).await;

        assert_eq!(
            findings,
            vec![BrpFinding::MissingInBrp {
                field: BrpCheckedField::Initials
            }]
        );
    }

    #[tokio::test]
    async fn a_candidate_without_a_gender_is_not_compared_on_it() {
        let (mut person, bsn) = candidate();
        person.personal_data.gender = None;
        let mut record = matching_record(&bsn);
        record["geslacht"]["code"] = json!("M");

        let findings = findings_for_record(&person, record).await;

        assert!(
            findings.is_empty(),
            "expected no findings, got {findings:?}"
        );
    }

    #[tokio::test]
    async fn an_unknown_bsn_is_reported() {
        let (person, _) = candidate();

        let findings = findings_for_record(&person, matching_record("999992806")).await;

        assert_eq!(findings, vec![BrpFinding::BsnUnknown]);
    }

    #[tokio::test]
    async fn a_bsn_matching_more_than_one_person_is_reported() {
        let (person, bsn) = candidate();
        let stub = BrpStub::serving(vec![matching_record(&bsn), matching_record(&bsn)]).await;

        let results = stub
            .client
            .verify_batch(std::slice::from_ref(&person), &ElectionConfig::EK27)
            .await
            .unwrap();

        assert_eq!(results[0].1, vec![BrpFinding::BsnNotUnique]);
    }

    #[tokio::test]
    async fn a_candidate_without_a_bsn_is_reported_without_asking_the_brp() {
        let mut without = sample_person(PersonId::new());
        without.personal_data.bsn = None;
        let mut none_confirmed = sample_person(PersonId::new());
        none_confirmed.personal_data.bsn = Some(BsnOrNoneConfirmed::NoneConfirmed);

        let stub = BrpStub::serving(Vec::new()).await;
        let results = stub
            .client
            .verify_batch(
                &[without.clone(), none_confirmed.clone()],
                &ElectionConfig::EK27,
            )
            .await
            .unwrap();

        assert_eq!(
            stub.query_count(),
            0,
            "no candidate could be looked up, so no request should be sent"
        );
        let findings: Vec<_> = results.into_iter().collect();
        assert!(findings.contains(&(without.id, vec![BrpFinding::BsnMissing])));
        assert!(findings.contains(&(none_confirmed.id, vec![BrpFinding::BsnNoneConfirmed])));
    }

    #[tokio::test]
    async fn eligibility_is_checked_against_the_brp() {
        let (person, bsn) = candidate();

        let mut deceased = matching_record(&bsn);
        deceased["overlijden"] = json!({ "datum": { "datum": "2026-01-31" } });
        assert!(
            findings_for_record(&person, deceased)
                .await
                .contains(&BrpFinding::Deceased {
                    date_of_death: "31-01-2026".to_string()
                })
        );

        let mut not_dutch = matching_record(&bsn);
        not_dutch["nationaliteiten"] = json!([{ "nationaliteit": { "code": "0031" } }]);
        assert!(
            findings_for_record(&person, not_dutch)
                .await
                .contains(&BrpFinding::NotDutch)
        );

        let mut excluded = matching_record(&bsn);
        excluded["uitsluitingKiesrecht"]["uitgeslotenVanKiesrecht"] = json!(true);
        assert!(
            findings_for_record(&person, excluded)
                .await
                .contains(&BrpFinding::ExcludedFromSuffrage)
        );
    }

    #[tokio::test]
    async fn a_candidate_who_is_too_young_to_be_elected_is_reported() {
        let (mut person, bsn) = candidate();
        // A day past the last date of birth that still makes a candidate
        // eligible for this election.
        let too_young = ElectionConfig::EK27.eligible_date_of_birth() + chrono::Days::new(1);
        person.personal_data.date_of_birth = Some(DateOfBirth::from(too_young));

        let mut record = matching_record(&bsn);
        record["geboorte"]["datum"]["datum"] = json!(too_young.format(BRP_DATE_FORMAT).to_string());

        let findings = findings_for_record(&person, record).await;

        assert_eq!(
            findings,
            vec![BrpFinding::Underage {
                date_of_birth: format_date(&DateOfBirth::from(too_young)),
            }]
        );
    }

    #[tokio::test]
    async fn a_missing_date_of_birth_means_the_age_could_not_be_established() {
        let (person, bsn) = candidate();
        let mut record = matching_record(&bsn);
        record["geboorte"] = json!({});

        let findings = findings_for_record(&person, record).await;

        assert_eq!(findings, vec![BrpFinding::AgeUnknown]);
    }

    #[tokio::test]
    async fn each_residence_without_a_woonplaats_gets_its_own_finding() {
        let (person, bsn) = candidate();

        for (residence_type, expected) in [
            ("VerblijfplaatsBuitenland", BrpFinding::ResidenceAbroad),
            ("Locatie", BrpFinding::ResidenceWithoutAddress),
            ("VerblijfplaatsOnbekend", BrpFinding::ResidenceUnknown),
            // A `type` this application does not know is not silently skipped.
            ("SomethingNew", BrpFinding::ResidenceUnknown),
        ] {
            let mut record = matching_record(&bsn);
            record["verblijfplaats"] = json!({ "type": residence_type });

            let findings = findings_for_record(&person, record).await;

            assert_eq!(
                findings,
                vec![expected],
                "unexpected findings for verblijfplaats {residence_type}"
            );
        }
    }

    #[tokio::test]
    async fn a_batch_is_looked_up_in_a_single_request() {
        let candidates: Vec<Person> = valid_bsns(BRP_BSN_BATCH_SIZE)
            .into_iter()
            .map(|bsn| {
                let mut person = sample_person(PersonId::new());
                person.personal_data.bsn = Some(BsnOrNoneConfirmed::Bsn(bsn));
                person
            })
            .collect();

        let stub = BrpStub::serving(Vec::new()).await;
        let results = stub
            .client
            .verify_batch(&candidates, &ElectionConfig::EK27)
            .await
            .unwrap();

        let query = stub.only_query();
        assert_eq!(
            query["burgerservicenummer"].as_array().map(Vec::len),
            Some(BRP_BSN_BATCH_SIZE),
            "all candidates in the batch should travel in one request"
        );
        assert_eq!(query["type"], json!("RaadpleegMetBurgerservicenummer"));
        // None of them is in the stub's list.
        assert_eq!(results.len(), BRP_BSN_BATCH_SIZE);
        assert!(results.iter().all(|(_, f)| f == &[BrpFinding::BsnUnknown]));
    }

    #[tokio::test]
    async fn a_batch_larger_than_the_brp_accepts_is_refused() {
        let candidates: Vec<Person> = (0..=BRP_BSN_BATCH_SIZE)
            .map(|_| sample_person(PersonId::new()))
            .collect();

        let stub = BrpStub::serving(Vec::new()).await;
        let result = stub
            .client
            .verify_batch(&candidates, &ElectionConfig::EK27)
            .await;

        assert!(matches!(result, Err(AppError::BrpError(_))), "{result:?}");
    }

    #[tokio::test]
    async fn an_unreachable_brp_is_an_error_rather_than_an_empty_result() {
        let (person, _) = candidate();
        // Port 1 on loopback refuses connections.
        let client = BrpClient::new_for_test("http://127.0.0.1:1");

        let result = client
            .verify_batch(std::slice::from_ref(&person), &ElectionConfig::EK27)
            .await;

        assert!(
            result.is_err(),
            "an unreachable BRP must not look like a candidate with no findings"
        );
    }

    #[tokio::test]
    async fn a_brp_error_response_is_an_error() {
        let router = Router::new().route(
            &format!("/{}", constants::BRP_PERSONS_ENDPOINT),
            post(|| async { axum::http::StatusCode::SERVICE_UNAVAILABLE }),
        );
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        let server = tokio::spawn(async move { axum::serve(listener, router).await.unwrap() });

        let (person, _) = candidate();
        let client = BrpClient::new_for_test(&format!("http://{addr}"));
        let result = client
            .verify_batch(std::slice::from_ref(&person), &ElectionConfig::EK27)
            .await;

        server.abort();
        assert!(result.is_err(), "a 503 from the BRP must not be ignored");
    }

    /// Smoke test against the real mock, which is not started by `cargo test`.
    /// Run with `docker compose up -d personen-mock` and
    /// `cargo test -- --ignored brp_mock`.
    #[tokio::test]
    #[ignore = "requires the personen-mock container: docker compose up -d personen-mock"]
    async fn brp_mock_answers_a_query() {
        let client = BrpClient::new_for_test("http://localhost:5010");
        let query = BrpQuery::ConsultWithBsn {
            bsn: vec!["999993653".parse().unwrap()],
            fields: CANDIDATE_FIELDS.to_vec(),
        };

        let persons = client.get_persons(&query).await.expect("mock answers");

        assert_eq!(persons.len(), 1);
        assert_eq!(persons[0].bsn.as_deref(), Some("999993653"));
    }
}
