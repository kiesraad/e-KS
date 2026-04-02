use serde::{Deserialize, Serialize};

use crate::core::AnyLocale;

/// Electoral districts used for nomination and submission flows.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum ElectoralDistrict {
    DR,
    FL,
    FR,
    GE,
    GR,
    LI,
    NB,
    NH,
    OV,
    UT,
    ZE,
    ZH,
    BO,
    SE,
    SA,
    KN,
}

impl ElectoralDistrict {
    pub fn ek2027() -> &'static [Self] {
        &[
            Self::DR,
            Self::FL,
            Self::FR,
            Self::GE,
            Self::GR,
            Self::LI,
            Self::NB,
            Self::NH,
            Self::OV,
            Self::UT,
            Self::ZE,
            Self::ZH,
            Self::BO,
            Self::SE,
            Self::SA,
            Self::KN,
        ]
    }

    pub fn title(&self, locale: AnyLocale) -> &'static str {
        match (self, locale) {
            (Self::DR, AnyLocale::Nl | AnyLocale::En) => "Drenthe",
            (Self::DR, AnyLocale::Fry) => "Drinte",
            (Self::FL, AnyLocale::Nl | AnyLocale::En) => "Flevoland",
            (Self::FL, AnyLocale::Fry) => "Flevolân",
            (Self::FR, AnyLocale::Nl | AnyLocale::En) => "Friesland",
            (Self::FR, AnyLocale::Fry) => "Fryslân",
            (Self::GE, AnyLocale::Nl | AnyLocale::En) => "Gelderland",
            (Self::GE, AnyLocale::Fry) => "Gelderlân",
            (Self::GR, AnyLocale::Nl | AnyLocale::En) => "Groningen",
            (Self::GR, AnyLocale::Fry) => "Grinslân",
            (Self::LI, AnyLocale::Nl | AnyLocale::En) => "Limburg",
            (Self::LI, AnyLocale::Fry) => "Limboarch",
            (Self::NB, AnyLocale::Nl) => "Noord-Brabant",
            (Self::NB, AnyLocale::En) => "North Brabant",
            (Self::NB, AnyLocale::Fry) => "Noard-Brabân",
            (Self::NH, AnyLocale::Nl) => "Noord-Holland",
            (Self::NH, AnyLocale::En) => "North Holland",
            (Self::NH, AnyLocale::Fry) => "Noard-Hollân",
            (Self::OV, AnyLocale::Nl | AnyLocale::En) => "Overijssel",
            (Self::OV, AnyLocale::Fry) => "Oerisel",
            (Self::UT, AnyLocale::Nl | AnyLocale::En) => "Utrecht",
            (Self::UT, AnyLocale::Fry) => "Utert",
            (Self::ZE, AnyLocale::Nl | AnyLocale::En) => "Zeeland",
            (Self::ZE, AnyLocale::Fry) => "Seelân",
            (Self::ZH, AnyLocale::Nl) => "Zuid-Holland",
            (Self::ZH, AnyLocale::En) => "South Holland",
            (Self::ZH, AnyLocale::Fry) => "Súd-Hollân",
            (Self::BO, AnyLocale::Nl) => "Kiescollege Bonaire",
            (Self::BO, AnyLocale::En) => "Electoral College Bonaire",
            (Self::BO, AnyLocale::Fry) => "Kieskolleezje Bonêre",
            (Self::SE, AnyLocale::Nl) => "Kiescollege Sint Eustatius",
            (Self::SE, AnyLocale::En) => "Electoral College Sint Eustatius",
            (Self::SE, AnyLocale::Fry) => "Kieskolleezje Sint Eustaasjus",
            (Self::SA, AnyLocale::Nl) => "Kiescollege Saba",
            (Self::SA, AnyLocale::En) => "Electoral College Saba",
            (Self::SA, AnyLocale::Fry) => "Kieskolleezje Saba",
            (Self::KN, AnyLocale::Nl) => "Kiescollege Niet-Ingezetenen",
            (Self::KN, AnyLocale::En) => "Electoral College Non-Residents",
            (Self::KN, AnyLocale::Fry) => "Kieskolleezje Net-Ynwenners",
        }
    }

    pub fn code(&self) -> &str {
        match self {
            Self::DR => "DR",
            Self::FL => "FL",
            Self::FR => "FR",
            Self::GE => "GE",
            Self::GR => "GR",
            Self::LI => "LI",
            Self::NB => "NB",
            Self::NH => "NH",
            Self::OV => "OV",
            Self::UT => "UT",
            Self::ZE => "ZE",
            Self::ZH => "ZH",
            Self::BO => "BO",
            Self::SE => "SE",
            Self::SA => "SA",
            Self::KN => "KN",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn district_title_and_code_match() {
        assert_eq!(ElectoralDistrict::UT.code(), "UT");
        assert_eq!(ElectoralDistrict::UT.title(AnyLocale::Nl), "Utrecht");
        assert_eq!(ElectoralDistrict::UT.title(AnyLocale::Fry), "Utert");
    }

    #[test]
    fn electoral_districts_include_expected_code() {
        let districts = ElectoralDistrict::ek2027();
        assert!(districts.contains(&ElectoralDistrict::UT));
        assert_eq!(districts.len(), 16);
    }
}
