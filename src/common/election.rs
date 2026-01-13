use serde::{Deserialize, Serialize};
use sqlx::postgres::{PgHasArrayType, PgTypeInfo};

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, sqlx::Type, Serialize, Deserialize,
)]
#[sqlx(no_pg_array)]
#[sqlx(type_name = "electoral_district")]
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

impl PgHasArrayType for ElectoralDistrict {
    fn array_type_info() -> PgTypeInfo {
        PgTypeInfo::array_of("electoral_district")
    }
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

    pub fn title(&self) -> &str {
        match self {
            Self::DR => "Drenthe",
            Self::FL => "Flevoland",
            Self::FR => "Friesland",
            Self::GE => "Gelderland",
            Self::GR => "Groningen",
            Self::LI => "Limburg",
            Self::NB => "Noord-Brabant",
            Self::NH => "Noord-Holland",
            Self::OV => "Overijssel",
            Self::UT => "Utrecht",
            Self::ZE => "Zeeland",
            Self::ZH => "Zuid-Holland",
            Self::BO => "Kiescollege Bonaire",
            Self::SE => "Kiescollege Sint Eustatius",
            Self::SA => "Kiescollege Saba",
            Self::KN => "Kiescollege Niet-Ingezetenen",
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

#[derive(Debug, Clone, Copy)]
pub enum ElectionConfig {
    EK2027,
}

impl ElectionConfig {
    pub fn title(&self) -> &str {
        match self {
            Self::EK2027 => "Eerste Kamerverkiezing der Staten-Generaal 2027",
        }
    }

    pub fn short_title(&self) -> &str {
        match self {
            Self::EK2027 => "Eerste Kamer 2027",
        }
    }

    pub fn electoral_districts(&self) -> &'static [ElectoralDistrict] {
        match self {
            Self::EK2027 => ElectoralDistrict::ek2027(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn electoral_districts_include_expected_code() {
        let districts = ElectoralDistrict::ek2027();
        assert!(districts.contains(&ElectoralDistrict::UT));
        assert_eq!(districts.len(), 16);
    }

    #[test]
    fn district_title_and_code_match() {
        assert_eq!(ElectoralDistrict::UT.code(), "UT");
        assert_eq!(ElectoralDistrict::UT.title(), "Utrecht");
    }

    #[test]
    fn election_config_exposes_districts() {
        let districts = ElectionConfig::EK2027.electoral_districts();
        assert!(districts.contains(&ElectoralDistrict::NH));
    }
}
