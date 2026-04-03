use serde::{Deserialize, Serialize};

use crate::core::AnyLocale;

/// Regions for the elections of the provincial council
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Province {
    GR,
    FR,
    DR,
    OV,
    FL,
    GE,
    UT,
    NH,
    ZH,
    ZE,
    NB,
    LI,
}

/// Electoral districts used for nomination and submission flows.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum ElectoralDistrict {
    GR,
    PsGroningen,
    FR,
    PsLeeuwarden,
    DR,
    PsAssen,
    OV,
    PsZwolle,
    FL,
    PsLelystad,
    GE,
    PsNijmegen,
    PsArnhem,
    UT,
    PsUtrecht,
    NH,
    PsAmsterdam,
    PsHaarlem,
    PsDenHelder,
    ZH,
    PsDenHaag,
    PsRotterdam,
    PsDordrecht,
    PsLeiden,
    ZE,
    PsMiddelburg,
    NB,
    PsTilburg,
    PsDenBosch,
    LI,
    PsMaastricht,
    PsVenlo,
    BO,
    SE,
    SA,
    KN,
}

impl ElectoralDistrict {
    pub fn ek27() -> &'static [Self] {
        &[
            Self::GR,
            Self::FR,
            Self::DR,
            Self::OV,
            Self::FL,
            Self::GE,
            Self::UT,
            Self::NH,
            Self::ZH,
            Self::ZE,
            Self::NB,
            Self::LI,
            Self::BO,
            Self::SE,
            Self::SA,
            Self::KN,
        ]
    }

    pub fn title(&self, locale: AnyLocale) -> &'static str {
        match (self, locale) {
            (Self::GR, AnyLocale::Nl | AnyLocale::En) => "Groningen",
            (Self::GR, AnyLocale::Fry) => "Grinslân",
            (Self::PsGroningen, AnyLocale::Nl | AnyLocale::En | AnyLocale::Fry) => "Groningen",
            (Self::FR, AnyLocale::Nl | AnyLocale::En) => "Friesland",
            (Self::FR, AnyLocale::Fry) => "Fryslân",
            (Self::PsLeeuwarden, AnyLocale::Nl | AnyLocale::En) => "Leeuwarden",
            (Self::PsLeeuwarden, AnyLocale::Fry) => "Ljouwert",
            (Self::DR, AnyLocale::Nl | AnyLocale::En) => "Drenthe",
            (Self::DR, AnyLocale::Fry) => "Drinte",
            (Self::PsAssen, AnyLocale::Nl | AnyLocale::En | AnyLocale::Fry) => "Assen",
            (Self::OV, AnyLocale::Nl | AnyLocale::En) => "Overijssel",
            (Self::OV, AnyLocale::Fry) => "Oerisel",
            (Self::PsZwolle, AnyLocale::Nl | AnyLocale::En | AnyLocale::Fry) => "Zwolle",
            (Self::FL, AnyLocale::Nl | AnyLocale::En) => "Flevoland",
            (Self::FL, AnyLocale::Fry) => "Flevolân",
            (Self::PsLelystad, AnyLocale::Nl | AnyLocale::En | AnyLocale::Fry) => "Lelystad",
            (Self::GE, AnyLocale::Nl | AnyLocale::En) => "Gelderland",
            (Self::GE, AnyLocale::Fry) => "Gelderlân",
            (Self::PsNijmegen, AnyLocale::Nl | AnyLocale::En | AnyLocale::Fry) => "Nijmegen",
            (Self::PsArnhem, AnyLocale::Nl | AnyLocale::En | AnyLocale::Fry) => "Arnhem",
            (Self::UT, AnyLocale::Nl | AnyLocale::En) => "Utrecht",
            (Self::UT, AnyLocale::Fry) => "Utert",
            (Self::PsUtrecht, AnyLocale::Nl | AnyLocale::En | AnyLocale::Fry) => "Utrecht",
            (Self::NH, AnyLocale::Nl) => "Noord-Holland",
            (Self::NH, AnyLocale::En) => "North Holland",
            (Self::NH, AnyLocale::Fry) => "Noard-Hollân",
            (Self::PsAmsterdam, AnyLocale::Nl | AnyLocale::En | AnyLocale::Fry) => "Amsterdam",
            (Self::PsHaarlem, AnyLocale::Nl | AnyLocale::En | AnyLocale::Fry) => "Haarlem",
            (Self::PsDenHelder, AnyLocale::Nl | AnyLocale::En | AnyLocale::Fry) => "Den Helder",
            (Self::ZH, AnyLocale::Nl) => "Zuid-Holland",
            (Self::ZH, AnyLocale::En) => "South Holland",
            (Self::ZH, AnyLocale::Fry) => "Súd-Hollân",
            (Self::PsDenHaag, AnyLocale::Nl | AnyLocale::Fry) => "'s-Gravenhage",
            (Self::PsDenHaag, AnyLocale::En) => "The Hague",
            (Self::PsRotterdam, AnyLocale::Nl | AnyLocale::En | AnyLocale::Fry) => "Rotterdam",
            (Self::PsDordrecht, AnyLocale::Nl | AnyLocale::En | AnyLocale::Fry) => "Dordrecht",
            (Self::PsLeiden, AnyLocale::Nl | AnyLocale::En | AnyLocale::Fry) => "Leiden",
            (Self::ZE, AnyLocale::Nl | AnyLocale::En) => "Zeeland",
            (Self::ZE, AnyLocale::Fry) => "Seelân",
            (Self::PsMiddelburg, AnyLocale::Nl | AnyLocale::En | AnyLocale::Fry) => "Middelburg",
            (Self::NB, AnyLocale::Nl) => "Noord-Brabant",
            (Self::NB, AnyLocale::En) => "North Brabant",
            (Self::NB, AnyLocale::Fry) => "Noard-Brabân",
            (Self::PsTilburg, AnyLocale::Nl | AnyLocale::En | AnyLocale::Fry) => "Tilburg",
            (Self::PsDenBosch, AnyLocale::Nl | AnyLocale::En | AnyLocale::Fry) => {
                "'s-Hertogenbosch"
            }

            (Self::LI, AnyLocale::Nl | AnyLocale::En) => "Limburg",
            (Self::LI, AnyLocale::Fry) => "Limboarch",
            (Self::PsMaastricht, AnyLocale::Nl | AnyLocale::En | AnyLocale::Fry) => "Maastricht",
            (Self::PsVenlo, AnyLocale::Nl | AnyLocale::En | AnyLocale::Fry) => "Venlo",
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
            Self::GR => "GR",
            Self::PsGroningen => "GRQ",
            Self::FR => "FR",
            Self::PsLeeuwarden => "LWR",
            Self::DR => "DR",
            Self::PsAssen => "ASS",
            Self::OV => "OV",
            Self::PsZwolle => "ZWO",
            Self::FL => "FL",
            Self::PsLelystad => "LEY",
            Self::GE => "GE",
            Self::PsNijmegen => "NIJ",
            Self::PsArnhem => "ARN",
            Self::UT => "UT",
            Self::PsUtrecht => "UTC",
            Self::NH => "NH",
            Self::PsAmsterdam => "AMS",
            Self::PsHaarlem => "HAA",
            Self::PsDenHelder => "DHR",
            Self::ZH => "ZH",
            Self::PsDenHaag => "HAG",
            Self::PsRotterdam => "RTM",
            Self::PsDordrecht => "DOR",
            Self::PsLeiden => "LID",
            Self::ZE => "ZE",
            Self::PsMiddelburg => "MDL",
            Self::NB => "NB",
            Self::PsTilburg => "TLB",
            Self::PsDenBosch => "HTB",
            Self::LI => "LI",
            Self::PsMaastricht => "MST",
            Self::PsVenlo => "VEN",
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

        assert_eq!(ElectoralDistrict::PsArnhem.code(), "ARN");
        assert_eq!(ElectoralDistrict::PsArnhem.title(AnyLocale::Nl), "Arnhem");
        assert_eq!(
            ElectoralDistrict::PsDenHaag.title(AnyLocale::Nl),
            "'s-Gravenhage"
        );
        assert_eq!(
            ElectoralDistrict::PsDenHaag.title(AnyLocale::En),
            "The Hague"
        );
    }

    #[test]
    fn electoral_districts_include_expected_code() {
        let districts = ElectoralDistrict::ek27();
        assert!(districts.contains(&ElectoralDistrict::UT));
        assert_eq!(districts.len(), 16);
    }
}
