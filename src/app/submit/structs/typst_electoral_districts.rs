use serde::Serialize;

use crate::{ElectionConfig, candidate_lists::CandidateList, core::ModelLocale};

#[derive(Debug, Serialize)]
#[serde(tag = "tag", content = "districts")]
pub enum TypstElectoralDistricts {
    All,
    Some(Vec<String>),
}

impl TypstElectoralDistricts {
    pub fn from(
        list: &CandidateList,
        election_config: &ElectionConfig,
        locale: ModelLocale,
    ) -> Self {
        if list.contains_all_districts(election_config) {
            TypstElectoralDistricts::All
        } else {
            TypstElectoralDistricts::Some(
                list.electoral_districts
                    .iter()
                    .map(|d| d.title(locale.into()).to_string())
                    .collect(),
            )
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
            TypstElectoralDistricts::from(&list, &election, ModelLocale::Fry),
            TypstElectoralDistricts::All
        ));
    }

    #[test]
    fn electoral_districts_from_partial_list_returns_titles() {
        let election = ElectionConfig::EK2027;
        let list = CandidateList {
            electoral_districts: vec![ElectoralDistrict::UT, ElectoralDistrict::NH],
            ..Default::default()
        };

        match TypstElectoralDistricts::from(&list, &election, ModelLocale::Nl) {
            TypstElectoralDistricts::Some(districts) => {
                assert_eq!(
                    districts,
                    vec!["Utrecht".to_string(), "Noord-Holland".to_string()]
                );
            }
            TypstElectoralDistricts::All => panic!("expected Some districts"),
        }
        match TypstElectoralDistricts::from(&list, &election, ModelLocale::Fry) {
            TypstElectoralDistricts::Some(districts) => {
                assert_eq!(
                    districts,
                    vec!["Utert".to_string(), "Noard-Hollân".to_string()]
                );
            }
            TypstElectoralDistricts::All => panic!("expected Some districts"),
        }
    }
}
