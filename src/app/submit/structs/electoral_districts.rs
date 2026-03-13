use std::fmt::{self, Display, Formatter};

use serde::Serialize;

use crate::{ElectionConfig, candidate_lists::CandidateList, core::ModelLocale};

#[derive(Debug, Serialize)]
#[serde(tag = "tag", content = "districts")]
pub enum ElectoralDistricts {
    All,
    Some(Vec<String>),
}

impl ElectoralDistricts {
    pub fn from(
        list: &CandidateList,
        election_config: &ElectionConfig,
        locale: ModelLocale,
    ) -> Self {
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

impl Display for ElectoralDistricts {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            ElectoralDistricts::All => write!(f, "*"),
            ElectoralDistricts::Some(districts) => write!(
                f,
                "{}",
                districts
                    .iter()
                    .flat_map(|d| d.get(..2))
                    .collect::<Vec<_>>()
                    .join("-")
                    .as_str()
            ),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ElectoralDistrict;

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
}
