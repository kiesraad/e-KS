use chrono::{Datelike, NaiveDate, NaiveDateTime, NaiveTime};

use crate::{
    ElectoralDistrict,
    core::{
        AnyLocale, ElectionType, ModelLocale,
        election::{Province, PublicSession, WaterCouncil},
    },
};

super::define_elections! {
    EK27 {
        election_type: ElectionType::Ek,
        titles: {
            nl: "Eerste Kamerverkiezing der Staten-Generaal 2027",
            fry: "Earste Keamerferkiezings fan de Steaten-Generaal 2027",
            en: "Election of the Senate of the States General 2027",
        },
        electoral_districts: ElectoralDistrict::ek27(),
        nineteen_or_more_seats: true,
        frisian_export_allowed: false,
        eligible_date_of_birth: NaiveDate::from_ymd_opt(2014, 4, 20).unwrap(), // TODO: determine definitive date
        nomination_day_date: NaiveDate::from_ymd_opt(2027, 4, 20).unwrap(),
        // Estimated from EK 2023 planning (official 2027 planning not yet published)
        document_review_date: NaiveDate::from_ymd_opt(2027, 4, 25).unwrap(),
        omission_period_end_date: NaiveDate::from_ymd_opt(2027, 4, 29).unwrap(),
        public_session: PublicSession {
            location: "'s-Gravenhage",
            datetime: NaiveDateTime::new(
                NaiveDate::from_ymd_opt(2027, 5, 3).unwrap(),
                NaiveTime::from_hms_opt(17, 0, 0).unwrap(),
            ),
            chair: "",
            members: &[""],
        },
        election_date: NaiveDate::from_ymd_opt(2027, 5, 24).unwrap()
    },

    PS27(province: Province) {
        election_type: ElectionType::Ps,
        titles: {
            nl: "Provinciale Statenverkiezingen 2027",
            fry: "Provinsjale Steateferkiezings 2027",
            en: "Elections of the Provincial Council 2027",
        },
        electoral_districts: match province {
            Province::GR => &[ElectoralDistrict::PsGroningen],
            Province::FR => &[ElectoralDistrict::PsLeeuwarden],
            Province::DR => &[ElectoralDistrict::PsAssen],
            Province::OV => &[ElectoralDistrict::PsZwolle],
            Province::FL => &[ElectoralDistrict::PsLelystad],
            Province::GE => &[ElectoralDistrict::PsNijmegen, ElectoralDistrict::PsArnhem],
            Province::UT => &[ElectoralDistrict::PsUtrecht],
            Province::NH => &[ElectoralDistrict::PsAmsterdam, ElectoralDistrict::PsHaarlem, ElectoralDistrict::PsDenHelder],
            Province::ZH => &[ElectoralDistrict::PsDenHaag, ElectoralDistrict::PsRotterdam, ElectoralDistrict::PsDordrecht, ElectoralDistrict::PsLeiden],
            Province::ZE => &[ElectoralDistrict::PsMiddelburg],
            Province::NB => &[ElectoralDistrict::PsTilburg, ElectoralDistrict::PsDenBosch],
            Province::LI => &[ElectoralDistrict::PsMaastricht, ElectoralDistrict::PsVenlo],
        },
        nineteen_or_more_seats: true, // for this election, all provinces have >= 19 seats
        frisian_export_allowed: matches!(province, Province::FR),
        eligible_date_of_birth: NaiveDate::from_ymd_opt(2014, 2, 1).unwrap(), // TODO: determine definitive date
        nomination_day_date: NaiveDate::from_ymd_opt(2027, 2, 1).unwrap(),
        document_review_date: NaiveDate::from_ymd_opt(2027, 2, 2).unwrap(),
        omission_period_end_date: NaiveDate::from_ymd_opt(2027, 2, 4).unwrap(),
        public_session: PublicSession {
            location: "'s-Gravenhage",
            datetime: NaiveDateTime::new(
                NaiveDate::from_ymd_opt(2027, 2, 5).unwrap(),
                NaiveTime::from_hms_opt(17, 0, 0).unwrap(),
            ),
            chair: "",
            members: &[""],
        },
        election_date: NaiveDate::from_ymd_opt(2027, 3, 17).unwrap()
    },

    WS27(water_council: WaterCouncil) {
        election_type: ElectionType::Ws,
        titles: {
            nl: "Waterschapsverkiezingen 2027",
            fry: "Wetterskipsferkiezings 2027",
            en: "Elections of the Water Authority 2027",
        },
        electoral_districts: match water_council {
            WaterCouncil::Noorderzijlvest => &[ElectoralDistrict::WsNoorderzijlvest],
            WaterCouncil::Fryslan => &[ElectoralDistrict::WsFryslan],
            WaterCouncil::HunzeEnAas => &[ElectoralDistrict::WsHunzeEnAas],
            WaterCouncil::DrentsOverijsselseDelta => &[ElectoralDistrict::WsDrentsOverijsselseDelta],
            WaterCouncil::Vechtstromen => &[ElectoralDistrict::WsVechtstromen],
            WaterCouncil::ValleiEnVeluwe => &[ElectoralDistrict::WsValleiEnVeluwe],
            WaterCouncil::RijnEnIJssel => &[ElectoralDistrict::WsRijnEnIJssel],
            WaterCouncil::DeStichtseRijnlanden => &[ElectoralDistrict::WsDeStichtseRijnlanden],
            WaterCouncil::AmstelGooiEnVecht => &[ElectoralDistrict::WsAmstelGooiEnVecht],
            WaterCouncil::HollandsNoorderkwartier => &[ElectoralDistrict::WsHollandsNoorderkwartier],
            WaterCouncil::Rijnland => &[ElectoralDistrict::WsRijnland],
            WaterCouncil::Delfland => &[ElectoralDistrict::WsDelfland],
            WaterCouncil::SchielandEnDeKrimpenerwaard => &[ElectoralDistrict::WsSchielandEnDeKrimpenerwaard],
            WaterCouncil::Rivierenland => &[ElectoralDistrict::WsRivierenland],
            WaterCouncil::HollandseDelta => &[ElectoralDistrict::WsHollandseDelta],
            WaterCouncil::Scheldestromen => &[ElectoralDistrict::WsScheldestromen],
            WaterCouncil::BrabantseDelta => &[ElectoralDistrict::WsBrabantseDelta],
            WaterCouncil::DeDommel => &[ElectoralDistrict::WsDeDommel],
            WaterCouncil::AaEnMaas => &[ElectoralDistrict::WsAaEnMaas],
            WaterCouncil::Limburg => &[ElectoralDistrict::WsLimburg],
            WaterCouncil::Zuiderzeeland => &[ElectoralDistrict::WsZuiderzeeland],
        },
        nineteen_or_more_seats: true, // for this election, all councils have >= 19 seats
        frisian_export_allowed: matches!(water_council, WaterCouncil::Fryslan),
        eligible_date_of_birth: NaiveDate::from_ymd_opt(2014, 2, 1).unwrap(), // TODO: determine definitive date
        nomination_day_date: NaiveDate::from_ymd_opt(2027, 2, 1).unwrap(),
        document_review_date: NaiveDate::from_ymd_opt(2027, 2, 2).unwrap(),
        omission_period_end_date: NaiveDate::from_ymd_opt(2027, 2, 4).unwrap(),
        public_session: PublicSession {
            location: "'s-Gravenhage",
            datetime: NaiveDateTime::new(
                NaiveDate::from_ymd_opt(2027, 2, 5).unwrap(),
                NaiveTime::from_hms_opt(17, 0, 0).unwrap(),
            ),
            chair: "",
            members: &[""],
        },
        election_date: NaiveDate::from_ymd_opt(2027, 3, 17).unwrap()
    }
}

impl ElectionConfig {
    /// Stable ID for the election configuration, used in HKDF derivation.
    pub fn stable_id(&self) -> String {
        let code = self.code();

        if let Some(region_code) = self.region_code() {
            format!("{code}:{region_code}")
        } else {
            code.to_string()
        }
    }

    /// The election title to be followed by the phrase "Het gaat om de verkiezing van ...", as written on the models.
    ///
    /// Specifies the region, but not the year of the election.
    pub fn formal_title(&self, locale: ModelLocale) -> String {
        let region = || {
            self.region_title(AnyLocale::from(locale))
                .expect("region title required for this election type")
        };

        match (self.election_type(), locale) {
            (ElectionType::Tk, ModelLocale::Nl) => {
                "de Tweede Kamer der Staten-Generaal".to_string()
            }
            (ElectionType::Tk, ModelLocale::Fry) => {
                "de Twadde Keamer fan de Steaten-Generaal".to_string()
            }

            (ElectionType::Ek, ModelLocale::Nl) => {
                "de Eerste Kamer der Staten-Generaal".to_string()
            }
            (ElectionType::Ek, ModelLocale::Fry) => {
                "de Earste Keamer fan de Steaten-Generaal".to_string()
            }

            (ElectionType::Gr, ModelLocale::Nl) => {
                format!("de gemeenteraad van {}", region())
            }
            (ElectionType::Gr, ModelLocale::Fry) => {
                format!("de gemeenterie fan {}", region())
            }

            (ElectionType::Ps, ModelLocale::Nl) => {
                format!("de provinciale staten van {}", region())
            }
            (ElectionType::Ps, ModelLocale::Fry) => {
                format!("de Provinsjale Steaten fan {}", region())
            }

            (ElectionType::Ws, ModelLocale::Nl) => {
                format!("het algemeen bestuur van het waterschap {}", region())
            }
            (ElectionType::Ws, ModelLocale::Fry) => {
                format!("it algemien bestjoer fan it wetterskip {}", region())
            }

            (ElectionType::Ep, ModelLocale::Nl) => "het Europees Parlement".to_string(),
            (ElectionType::Ep, ModelLocale::Fry) => "het Europees Parlement".to_string(),

            (ElectionType::Kc, _) => todo!("Support electoral college regions"),
            (ElectionType::Er, _) => todo!("Support island regions"),
        }
    }

    /// The full formal election title including the region and year, as listed in the EML 210.
    ///
    /// E.g. "Verkiezing van de gemeenteraad van Voorne aan Zee 2026"
    pub fn full_formal_title(&self, locale: ModelLocale) -> String {
        format!(
            "{} {} {}",
            match locale {
                ModelLocale::Fry => "Ferkiezing fan",
                ModelLocale::Nl => "Verkiezing van",
            },
            self.formal_title(locale),
            self.election_date().year()
        )
    }

    /// Returns all concrete election configurations.
    pub fn all() -> Vec<ElectionConfig> {
        let mut configs = vec![ElectionConfig::EK27];
        configs.extend(Province::ALL.iter().map(|p| ElectionConfig::PS27(*p)));
        configs.extend(WaterCouncil::ALL.iter().map(|wc| ElectionConfig::WS27(*wc)));
        configs
    }

    /// Returns one representative `ElectionConfig` per election type, for the
    /// type-selector dropdown. Derived from `ElectionConfig::all()` so new
    /// election types are picked up automatically.
    pub fn type_options() -> Vec<ElectionConfig> {
        let mut seen = std::collections::HashSet::new();
        Self::all()
            .into_iter()
            .filter(|e| seen.insert(e.code()))
            .collect()
    }

    pub fn available_districts(
        &self,
        used_districts: Vec<ElectoralDistrict>,
    ) -> Vec<ElectoralDistrict> {
        self.electoral_districts()
            .iter()
            .filter(|d| !used_districts.contains(d))
            .cloned()
            .collect()
    }

    pub fn has_only_one_district(&self) -> bool {
        self.electoral_districts().len() == 1
    }
}

#[cfg(test)]
mod tests {
    use crate::Locale;

    use super::*;

    #[test]
    fn election_titles_are_correct() {
        assert!(ElectionConfig::EK27.title(AnyLocale::Nl).len() > 20);

        let election_type = ElectionConfig::EK27.election_type();
        assert!(election_type.title(Locale::Nl).len() > 20);
    }

    #[test]
    fn election_config_exposes_districts() {
        let districts = ElectionConfig::EK27.electoral_districts();
        assert!(districts.contains(&ElectoralDistrict::NH));

        let districts = ElectionConfig::PS27(Province::GE).electoral_districts();
        assert!(districts.contains(&ElectoralDistrict::PsNijmegen));

        let districts = ElectionConfig::WS27(WaterCouncil::AaEnMaas).electoral_districts();
        assert_eq!(districts, &[ElectoralDistrict::WsAaEnMaas]);
        let districts = ElectionConfig::WS27(WaterCouncil::Rivierenland).electoral_districts();
        assert_eq!(districts, &[ElectoralDistrict::WsRivierenland]);
        let districts = ElectionConfig::WS27(WaterCouncil::ValleiEnVeluwe).electoral_districts();
        assert_eq!(districts, &[ElectoralDistrict::WsValleiEnVeluwe]);
    }

    #[test]
    fn has_only_district() {
        assert!(ElectionConfig::PS27(Province::DR).has_only_one_district());
        assert!(!ElectionConfig::PS27(Province::GE).has_only_one_district());
    }

    #[test]
    fn type_options_contains_one_per_election_code() {
        let options = ElectionConfig::type_options();

        let codes: Vec<&str> = options.iter().map(|e| e.code()).collect();
        assert_eq!(codes, vec!["EK27", "PS27", "WS27"]);

        // No duplicate codes — each election type appears at most once.
        let mut sorted = codes.clone();
        sorted.sort();
        sorted.dedup();
        assert_eq!(sorted.len(), codes.len());
    }

    #[test]
    fn from_code_and_region_resolves_region_less_election() {
        assert_eq!(
            ElectionConfig::from_code_and_region("EK27", None),
            Some(ElectionConfig::EK27)
        );
    }

    #[test]
    fn from_code_and_region_ignores_region_for_region_less_election() {
        // A spurious region argument is ignored for elections that don't take one.
        assert_eq!(
            ElectionConfig::from_code_and_region("EK27", Some("anything")),
            Some(ElectionConfig::EK27)
        );
    }

    #[test]
    fn from_code_and_region_resolves_ps27_with_valid_province() {
        assert_eq!(
            ElectionConfig::from_code_and_region("PS27", Some("GR")),
            Some(ElectionConfig::PS27(Province::GR))
        );
    }

    #[test]
    fn from_code_and_region_resolves_ws27_with_valid_water_council() {
        assert_eq!(
            ElectionConfig::from_code_and_region("WS27", Some("WS-FRY")),
            Some(ElectionConfig::WS27(WaterCouncil::Fryslan))
        );
    }

    #[test]
    fn from_code_and_region_returns_none_when_region_required_but_missing() {
        assert_eq!(ElectionConfig::from_code_and_region("PS27", None), None);
        assert_eq!(ElectionConfig::from_code_and_region("WS27", None), None);
    }

    #[test]
    fn from_code_and_region_returns_none_for_invalid_region() {
        assert_eq!(
            ElectionConfig::from_code_and_region("PS27", Some("XX")),
            None
        );
        assert_eq!(
            ElectionConfig::from_code_and_region("WS27", Some("NotAWaterCouncil")),
            None
        );
    }

    #[test]
    fn from_code_and_region_returns_none_for_unknown_code() {
        assert_eq!(
            ElectionConfig::from_code_and_region("ZZ99", Some("GR")),
            None
        );
        assert_eq!(ElectionConfig::from_code_and_region("", None), None);
    }

    #[test]
    fn formal_title() {
        assert_eq!(
            ElectionConfig::EK27.formal_title(ModelLocale::Nl),
            "de Eerste Kamer der Staten-Generaal"
        );
        assert_eq!(
            ElectionConfig::EK27.formal_title(ModelLocale::Fry),
            "de Earste Keamer fan de Steaten-Generaal"
        );
        assert_eq!(
            ElectionConfig::EK27.full_formal_title(ModelLocale::Nl),
            "Verkiezing van de Eerste Kamer der Staten-Generaal 2027"
        );
        assert_eq!(
            ElectionConfig::EK27.full_formal_title(ModelLocale::Fry),
            "Ferkiezing fan de Earste Keamer fan de Steaten-Generaal 2027"
        );

        assert_eq!(
            ElectionConfig::PS27(Province::DR).formal_title(ModelLocale::Nl),
            "de provinciale staten van Drenthe"
        );
        assert_eq!(
            ElectionConfig::PS27(Province::DR).full_formal_title(ModelLocale::Nl),
            "Verkiezing van de provinciale staten van Drenthe 2027"
        );

        assert_eq!(
            ElectionConfig::WS27(WaterCouncil::Fryslan).formal_title(ModelLocale::Fry),
            "it algemien bestjoer fan it wetterskip Fryslân"
        );
        assert_eq!(
            ElectionConfig::WS27(WaterCouncil::Fryslan).full_formal_title(ModelLocale::Fry),
            "Ferkiezing fan it algemien bestjoer fan it wetterskip Fryslân 2027"
        );
    }
}
