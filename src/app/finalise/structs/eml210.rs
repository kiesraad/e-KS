use chrono::Datelike;
use eml_nl::{
    common::{
        AuthorityIdentifier, CandidateIdentifier, CountryNameCode, CreatedByAuthority, FirstName,
        LastName, ListData, ListDataContest, ManagingAuthority, NameLineInitials, NamePrefix,
        PersonName,
    },
    documents::{
        EML,
        candidate_lists::QualifyingAddress,
        nomination::{
            AgentIdentifier, Nomination, NominationAffiliation, NominationContestIdentifier,
            NominationElectionIdentifier, NominationNominate,
        },
    },
    io::EMLWrite,
    utils::{AffiliationType, AuthorityId, CandidateId, ContestId, ElectionId, StringValue},
};

use crate::{
    AnyLocale, AppError, AppStore, ElectionConfig,
    candidate_lists::FullCandidateList,
    candidates::Candidate,
    common::{Address, BsnOrNoneConfirmed, DutchAddress, FullName, Gender},
    core::{ElectionType, ModelLocale},
    list_submitters::ListSubmitter,
    persons::Representative,
    political_groups::PoliticalGroup,
    utils::slugify_teletex,
};

impl From<ElectionType> for eml_nl::utils::ElectionCategory {
    fn from(value: ElectionType) -> Self {
        match value {
            ElectionType::Tk => eml_nl::utils::ElectionCategory::TK,
            ElectionType::Ek => eml_nl::utils::ElectionCategory::EK,
            ElectionType::Gr => eml_nl::utils::ElectionCategory::GR,
            ElectionType::Ps => eml_nl::utils::ElectionCategory::PS,
            ElectionType::Ws => eml_nl::utils::ElectionCategory::AB,
            ElectionType::Ep => eml_nl::utils::ElectionCategory::EP,
            ElectionType::Kc => todo!("Kiescolleges don't have an official code yet in EML-NL"),
            ElectionType::Er => eml_nl::utils::ElectionCategory::ER,
        }
    }
}

impl From<&ElectionConfig> for eml_nl::utils::ElectionSubcategory {
    fn from(value: &ElectionConfig) -> Self {
        match value.election_type() {
            ElectionType::Tk => eml_nl::utils::ElectionSubcategory::TK,
            ElectionType::Ek => eml_nl::utils::ElectionSubcategory::EK,
            ElectionType::Gr => {
                if value.nineteen_or_more_seats() {
                    eml_nl::utils::ElectionSubcategory::GR2
                } else {
                    eml_nl::utils::ElectionSubcategory::GR1
                }
            }
            ElectionType::Ps => {
                if value.has_only_one_district() {
                    eml_nl::utils::ElectionSubcategory::PS1
                } else {
                    eml_nl::utils::ElectionSubcategory::PS2
                }
            }
            ElectionType::Ws => {
                if value.nineteen_or_more_seats() {
                    eml_nl::utils::ElectionSubcategory::AB2
                } else {
                    eml_nl::utils::ElectionSubcategory::AB1
                }
            }
            ElectionType::Ep => eml_nl::utils::ElectionSubcategory::EP,
            ElectionType::Kc => todo!("Kiescolleges don't have an official code yet in EML-NL"),
            ElectionType::Er => eml_nl::utils::ElectionSubcategory::ER1,
        }
    }
}

impl TryFrom<ElectionConfig> for eml_nl::documents::nomination::NominationElectionIdentifier {
    type Error = AppError;

    fn try_from(value: ElectionConfig) -> Result<Self, Self::Error> {
        let category = eml_nl::utils::ElectionCategory::from(value.election_type());
        let year = value.election_date().year();

        let id = if let Some(region) = value.region_title(crate::AnyLocale::Nl) {
            format!(
                "{}{}_{}",
                category.to_eml_value(),
                year,
                slugify_teletex(region, false)
            )
        } else {
            format!("{}{}", category.to_eml_value(), year)
        };

        Ok(NominationElectionIdentifier::builder()
            .name(value.full_formal_title(crate::core::ModelLocale::Nl))
            .category(category)
            .subcategory(&value)
            .election_date(value.election_date())
            .nomination_date(value.nomination_day_date())
            .id(ElectionId::new(id)?)
            .build_for_nomination()?)
    }
}

impl From<&FullName> for eml_nl::common::PersonNameStructure {
    fn from(val: &FullName) -> Self {
        eml_nl::common::PersonNameStructure::new(PersonName {
            name_line_initials: Some(NameLineInitials::new(val.initials.to_string())),
            first_name: val
                .first_name
                .as_ref()
                .map(|n| FirstName::new(n.to_string())),
            name_prefix: val
                .last_name_prefix
                .as_ref()
                .map(|n| NamePrefix::new(n.to_string())),
            last_name: LastName::new(val.last_name.to_string()),
            person_name_type: None,
            code: None,
            name_details_key_ref: None,
        })
    }
}

impl From<&Address> for QualifyingAddress {
    fn from(address: &Address) -> QualifyingAddress {
        let locality = eml_nl::documents::candidate_lists::QualifyingAddressLocality::new(
            address
                .locality()
                .as_ref()
                .map(|loc| loc.to_string())
                .unwrap_or_default(),
        )
        .with_postal_code_option(address.postal_code())
        .with_address_line_option(address.address_line_1());

        QualifyingAddress::Locality(locality)
    }
}

impl From<&DutchAddress> for eml_nl::documents::nomination::LivingAddress {
    fn from(address: &DutchAddress) -> eml_nl::documents::nomination::LivingAddress {
        eml_nl::documents::nomination::LivingAddress::new(
            address
                .locality
                .as_ref()
                .map(|loc| loc.to_string())
                .unwrap_or_default(),
        )
    }
}

impl From<&Address> for eml_nl::documents::nomination::NominationContact {
    fn from(address: &Address) -> eml_nl::documents::nomination::NominationContact {
        eml_nl::documents::nomination::NominationContact {
            mailing_address: eml_nl::documents::nomination::MailingAddress {
                address: address.into(),
            },
        }
    }
}

impl From<&Representative> for eml_nl::documents::nomination::NominationAgent {
    fn from(representative: &Representative) -> eml_nl::documents::nomination::NominationAgent {
        eml_nl::documents::nomination::NominationAgent {
            role: Some("H10".to_string()),
            agent_identifier: AgentIdentifier::new(&representative.name),
            contact: Some((&Address::Dutch(representative.address.clone())).into()),
            living_address: (&representative.address).into(),
        }
    }
}

impl TryInto<eml_nl::documents::nomination::NominationCandidate> for &Candidate {
    type Error = AppError;

    fn try_into(self) -> Result<eml_nl::documents::nomination::NominationCandidate, Self::Error> {
        Ok(eml_nl::documents::nomination::NominationCandidate {
            identifier: CandidateIdentifier::new(
                CandidateId::from_u64(self.position as u64)
                    .map_err(|_| AppError::IncompleteData("candidate position is 0"))?,
            ),
            full_name: (&self.person.name).into(),
            date_of_birth: self
                .person
                .personal_data
                .date_of_birth
                .as_ref()
                .map(|n| StringValue::from_value((**n).into())),
            gender: StringValue::from_value(match self.person.personal_data.gender {
                None => eml_nl::utils::Gender::Unknown,
                Some(Gender::Female) => eml_nl::utils::Gender::Female,
                Some(Gender::Male) => eml_nl::utils::Gender::Male,
            }),
            qualifying_address: QualifyingAddress::new(
                self.person
                    .personal_data
                    .place_of_residence
                    .as_ref()
                    .ok_or(AppError::IncompleteData("missing place of residence"))?
                    .to_string(),
                match self
                    .person
                    .personal_data
                    .country
                    .as_ref()
                    .ok_or(AppError::IncompleteData("missing country"))?
                {
                    country if country.is_nl() => None,
                    country => Some(CountryNameCode::new(country.to_string())),
                },
            ),
            contact: self
                .person
                .lives_in_nl()
                .then(|| (&Address::Dutch(self.person.address.clone())).into()),
            agent: (!self.person.lives_in_nl())
                .then(|| self.person.representative.as_ref().map(Into::into))
                .flatten(),
            date_of_birth_annex: None,
            national_identification_number: match self.person.personal_data.bsn.as_ref() {
                Some(BsnOrNoneConfirmed::Bsn(bsn)) => Some(bsn.to_exposed_string().into()),
                _ => None,
            },
        })
    }
}

fn nomination_proposer(
    submitter: ListSubmitter,
    job_title: eml_nl::documents::nomination::NominationJobTitle,
    id: Option<Box<str>>,
) -> Result<eml_nl::documents::nomination::NominationProposer, AppError> {
    Ok(eml_nl::documents::nomination::NominationProposer {
        name: (&submitter.name).into(),
        contact: (&submitter.address).into(),
        job_title: StringValue::Parsed(job_title),
        id,
        living_address: None,
    })
}

pub struct Eml210(Nomination);

impl Eml210 {
    pub fn new(
        store: &AppStore,
        election: &ElectionConfig,
        political_group: &PoliticalGroup,
        list_id: crate::candidate_lists::CandidateListId,
        locale: ModelLocale,
    ) -> Result<Self, AppError> {
        let FullCandidateList { list, candidates } = FullCandidateList::get(store, list_id)?;

        let substitutes = store.get_substitute_submitters();
        let mut nominated = Vec::with_capacity(1 + substitutes.len());
        nominated.push(nomination_proposer(
            store.get_list_submitter(),
            eml_nl::documents::nomination::NominationJobTitle::Submitter,
            None,
        )?);

        for (i, sub) in substitutes.into_iter().enumerate() {
            nominated.push(nomination_proposer(
                sub,
                eml_nl::documents::nomination::NominationJobTitle::DeputySubmitter,
                Some((i + 1).to_string().into()),
            )?);
        }

        // ListData is additional data specifically for OSV, we can possibly change this in the future if necessary
        let list_data = ListData {
            // We always publish genders, but the individual candidates may leave the gender unspecified
            publish_gender: StringValue::Parsed(true),
            publication_language: Some(StringValue::from_value(match locale {
                crate::core::ModelLocale::Fry => eml_nl::utils::PublicationLanguage::Frisian,
                crate::core::ModelLocale::Nl => eml_nl::utils::PublicationLanguage::Dutch,
            })),
            belongs_to_set: None,
            belongs_to_combination: None,
            contests: list
                .electoral_districts
                .iter()
                .map(|d| {
                    Ok(ListDataContest::new(ContestId::new(d.region_number())?)
                        .with_name(d.title(AnyLocale::Nl)))
                })
                .collect::<Result<Vec<ListDataContest>, AppError>>()?,
        };

        let now = chrono::Utc::now();
        let nomination = Nomination::builder()
            .transaction_id(
                u64::try_from(store.current_event_id())
                    .map_err(|_| AppError::InternalServerError)?,
            )
            .managing_authority(
                ManagingAuthority::new(AuthorityIdentifier::new(AuthorityId::new("0000")?))
                    .with_created_by_authority(
                        CreatedByAuthority::new(AuthorityId::new("0000")?)
                            .with_name("De politieke partij"),
                    ),
            )
            .issue_date(now.date_naive())
            .creation_date_time(now)
            .election_identifier(NominationElectionIdentifier::try_from(*election)?)
            .contest_identifier(if election.has_only_one_district() {
                NominationContestIdentifier::new(ContestId::geen(), "")
            } else if list.contains_all_districts(election) {
                NominationContestIdentifier::new(ContestId::alle(), "")
            } else {
                // If there are multiple districts but this list is not linked to all districts,
                // we always choose the first district (to avoid collisions with other lists).
                // The full set of electoral districts can be found in the ListData.
                let district = list.electoral_districts[0];
                NominationContestIdentifier::new(
                    ContestId::new(district.region_number())?,
                    district.title(AnyLocale::Nl),
                )
            })
            .affiliation(NominationAffiliation {
                registered_name: political_group.pg_display_name()?.into(),
                affiliation_type: StringValue::from_value(AffiliationType::StandAloneList),
                list_data,
                candidates: candidates
                    .iter()
                    .map(|c| (&c.data).try_into())
                    .collect::<Result<Vec<_>, AppError>>()?,
            })
            .nominate(NominationNominate::new(nominated))
            .build()?;

        Ok(Self(nomination))
    }

    pub fn build(self) -> Result<Vec<u8>, AppError> {
        Ok(EML::from_nomination_doc(self.0).write_eml_root(true, true)?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    use crate::{
        AppError, AppStore, Context, ElectoralDistrict,
        candidate_lists::{CandidateListId, FullCandidateList},
        common::CountryCode,
        core::ModelLocale,
        list_submitters::ListSubmitterId,
        persons::{PersonId, Representative},
        test_utils::{
            sample_candidate_list, sample_dutch_address, sample_full_name, sample_list_submitter,
            sample_person,
        },
    };

    async fn create_sample_list(store: &AppStore) -> Result<FullCandidateList, AppError> {
        let list_id = CandidateListId::new();
        let mut list = sample_candidate_list(list_id);

        let person_id1 = PersonId::new();
        let mut sample_person1 = sample_person(person_id1);
        sample_person1.name.last_name = "Candidate I".parse().unwrap();
        sample_person1.create(store).await?;
        list.candidates.push(person_id1);

        let person_id2 = PersonId::new();
        let mut sample_person2 = sample_person(person_id2);
        sample_person2.name.last_name = "Candidate II".parse().unwrap();
        sample_person2.personal_data.bsn = Some("999995972".parse().unwrap());
        sample_person2.personal_data.country = CountryCode::from_str("BE").ok();
        sample_person2.personal_data.gender = None;

        sample_person2.representative = Some(Representative {
            name: sample_full_name(Some("Bob"), "Bouwer", Some("de"), "B."),
            address: sample_dutch_address("Nijmegen", "1234AB", "22", "c", "Bouwstraat"),
        });
        sample_person2.create(store).await?;
        list.candidates.push(person_id2);

        let mut submitter = sample_list_submitter(ListSubmitterId::new());
        submitter.name.last_name = "Submitter".parse().unwrap();
        submitter.update(store).await?;

        let mut sub_submitter1 = sample_list_submitter(ListSubmitterId::new());
        sub_submitter1.name.last_name = "Sub Submitter I".parse().unwrap();
        let mut sub_submitter2 = sample_list_submitter(ListSubmitterId::new());
        sub_submitter2.name.last_name = "Sub Submitter II".parse().unwrap();
        sub_submitter1.create_substitute(store).await?;
        sub_submitter2.create_substitute(store).await?;

        list.create(store).await?;

        FullCandidateList::get(store, list_id)
    }

    async fn check_eml(response: &str, expected: &str) {
        let stringify_nomination_data = |eml: eml_nl::documents::EML| {
            format!("{:?}", eml.as_nomination_doc().unwrap().nomination_data)
        };

        let received = stringify_nomination_data(response.parse().unwrap());
        let expected = stringify_nomination_data(expected.parse().unwrap());

        assert_eq!(received, expected, "received XML:\n{}", response);
    }

    #[tokio::test]
    async fn ek_export() {
        // setup
        let store = AppStore::new_for_test();
        let mut context = Context::new_test_without_db();
        context.election = ElectionConfig::EK27;
        let list = create_sample_list(&store).await.unwrap();

        // test
        let eml = Eml210::new(
            &store,
            &context.election,
            &store.get_political_group(),
            list.id(),
            ModelLocale::Nl,
        )
        .unwrap()
        .build()
        .unwrap();

        // verify
        check_eml(
            &String::from_utf8(eml).unwrap(),
            include_str!("../testdata/ek27.eml.xml"),
        )
        .await;
    }

    #[tokio::test]
    async fn ps1_export() {
        // setup
        let store = AppStore::new_for_test();
        let mut context = Context::new_test_without_db();
        context.election = ElectionConfig::PS27(crate::Province::GR);
        let mut list = create_sample_list(&store).await.unwrap();
        list.list.electoral_districts = vec![ElectoralDistrict::PsGroningen];
        list.list.update_districts(&store).await.unwrap();

        // test
        let eml = Eml210::new(
            &store,
            &context.election,
            &store.get_political_group(),
            list.id(),
            ModelLocale::Nl,
        )
        .unwrap()
        .build()
        .unwrap();

        // verify
        check_eml(
            &String::from_utf8(eml).unwrap(),
            include_str!("../testdata/ps27-1.eml.xml"),
        )
        .await;
    }

    #[tokio::test]
    async fn ps2_export() {
        // setup
        let store = AppStore::new_for_test();
        let mut context = Context::new_test_without_db();
        context.election = ElectionConfig::PS27(crate::Province::LI);
        let mut list = create_sample_list(&store).await.unwrap();
        list.list.electoral_districts =
            vec![ElectoralDistrict::PsMaastricht, ElectoralDistrict::PsVenlo];
        list.list.update_districts(&store).await.unwrap();

        // test
        let eml = Eml210::new(
            &store,
            &context.election,
            &store.get_political_group(),
            list.id(),
            ModelLocale::Nl,
        )
        .unwrap()
        .build()
        .unwrap();

        // verify
        check_eml(
            &String::from_utf8(eml).unwrap(),
            include_str!("../testdata/ps27-2.eml.xml"),
        )
        .await;
    }

    #[tokio::test]
    async fn ws_export() {
        // setup
        let store = AppStore::new_for_test();
        let mut context = Context::new_test_without_db();
        context.election = ElectionConfig::WS27(crate::WaterCouncil::Fryslan);
        let mut list = create_sample_list(&store).await.unwrap();
        list.list.electoral_districts = vec![ElectoralDistrict::WsFryslan];
        list.list.update_districts(&store).await.unwrap();

        // test
        let eml = Eml210::new(
            &store,
            &context.election,
            &store.get_political_group(),
            list.id(),
            ModelLocale::Fry,
        )
        .unwrap()
        .build()
        .unwrap();

        // verify
        check_eml(
            &String::from_utf8(eml).unwrap(),
            include_str!("../testdata/ws27.eml.xml"),
        )
        .await;
    }
}
