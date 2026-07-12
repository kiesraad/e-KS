use crate::{
    ElectionConfig, candidate_lists::CandidateList, core::ModelLocale,
    models::inputs::ElectoralDistricts,
};

impl ElectoralDistricts {
    pub fn from(
        list: &CandidateList,
        election_config: &ElectionConfig,
        locale: ModelLocale,
    ) -> Self {
        if election_config.has_only_one_district() {
            ElectoralDistricts::OnlyOne
        } else if list.contains_all_districts(election_config) {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{ElectoralDistrict, core::election::WaterCouncil};

    #[test]
    fn electoral_districts_from_full_list_returns_all() {
        let election = ElectionConfig::EK27;
        let list = CandidateList {
            electoral_districts: election.electoral_districts().to_vec(),
            ..Default::default()
        };

        assert_eq!(
            ElectoralDistricts::from(&list, &election, ModelLocale::Fry),
            ElectoralDistricts::All
        );
    }

    #[test]
    fn electoral_districts_from_partial_list_returns_titles() {
        let election = ElectionConfig::EK27;
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
            _ => panic!("expected Some districts"),
        }
        match ElectoralDistricts::from(&list, &election, ModelLocale::Fry) {
            ElectoralDistricts::Some(districts) => {
                assert_eq!(
                    districts,
                    vec!["Utert".to_string(), "Noard-Hollân".to_string()]
                );
            }
            _ => panic!("expected Some districts"),
        }
    }

    #[test]
    fn electoral_districts_with_only_one_district_returns_only_one() {
        let election = ElectionConfig::WS27(WaterCouncil::RijnEnIJssel);
        let list = CandidateList {
            ..Default::default()
        };

        let district = ElectoralDistricts::from(&list, &election, ModelLocale::Nl);

        assert_eq!(district, ElectoralDistricts::OnlyOne);
    }
}
