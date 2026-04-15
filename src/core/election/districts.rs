use serde::{Deserialize, Serialize};

use crate::core::AnyLocale;

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
    WsNoorderzijlvest,
    WsFryslan,
    WsHunzeEnAas,
    WsDrentsOverijsselseDelta,
    WsVechtstromen,
    WsValleiEnVeluwe,
    WsRijnEnIJssel,
    WsDeStichtseRijnlanden,
    WsAmstelGooiEnVecht,
    WsHollandsNoorderkwartier,
    WsRijnland,
    WsDelfland,
    WsSchielandEnDeKrimpenerwaard,
    WsRivierenland,
    WsHollandseDelta,
    WsScheldestromen,
    WsBrabantseDelta,
    WsDeDommel,
    WsAaEnMaas,
    WsLimburg,
    WsZuiderzeeland,
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

    /// Returns the serde variant name, used as form value so that
    /// `serde_urlencoded` can deserialize it back into `ElectoralDistrict`.
    pub fn serde_name(&self) -> String {
        serde_json::to_value(self)
            .and_then(serde_json::from_value)
            .expect("unit enum variant serializes to a string")
    }

    /// Returns (code, default_title, frisian_title) for each district.
    fn data(&self) -> (&'static str, &'static str, &'static str) {
        match self {
            Self::GR => ("GR", "Groningen", "Grinslân"),
            Self::PsGroningen => ("GRQ", "Groningen", "Groningen"),
            Self::FR => ("FR", "Friesland", "Fryslân"),
            Self::PsLeeuwarden => ("LWR", "Leeuwarden", "Ljouwert"),
            Self::DR => ("DR", "Drenthe", "Drinte"),
            Self::PsAssen => ("ASS", "Assen", "Assen"),
            Self::OV => ("OV", "Overijssel", "Oerisel"),
            Self::PsZwolle => ("ZWO", "Zwolle", "Zwolle"),
            Self::FL => ("FL", "Flevoland", "Flevolân"),
            Self::PsLelystad => ("LEY", "Lelystad", "Lelystad"),
            Self::GE => ("GE", "Gelderland", "Gelderlân"),
            Self::PsNijmegen => ("NIJ", "Nijmegen", "Nijmegen"),
            Self::PsArnhem => ("ARN", "Arnhem", "Arnhem"),
            Self::UT => ("UT", "Utrecht", "Utert"),
            Self::PsUtrecht => ("UTC", "Utrecht", "Utrecht"),
            Self::NH => ("NH", "Noord-Holland", "Noard-Hollân"),
            Self::PsAmsterdam => ("AMS", "Amsterdam", "Amsterdam"),
            Self::PsHaarlem => ("HAA", "Haarlem", "Haarlem"),
            Self::PsDenHelder => ("DHR", "Den Helder", "Den Helder"),
            Self::ZH => ("ZH", "Zuid-Holland", "Súd-Hollân"),
            Self::PsDenHaag => ("HAG", "'s-Gravenhage", "'s-Gravenhage"),
            Self::PsRotterdam => ("RTM", "Rotterdam", "Rotterdam"),
            Self::PsDordrecht => ("DOR", "Dordrecht", "Dordrecht"),
            Self::PsLeiden => ("LID", "Leiden", "Leiden"),
            Self::ZE => ("ZE", "Zeeland", "Seelân"),
            Self::PsMiddelburg => ("MDL", "Middelburg", "Middelburg"),
            Self::NB => ("NB", "Noord-Brabant", "Noard-Brabân"),
            Self::PsTilburg => ("TLB", "Tilburg", "Tilburg"),
            Self::PsDenBosch => ("HTB", "'s-Hertogenbosch", "'s-Hertogenbosch"),
            Self::LI => ("LI", "Limburg", "Limboarch"),
            Self::PsMaastricht => ("MST", "Maastricht", "Maastricht"),
            Self::PsVenlo => ("VEN", "Venlo", "Venlo"),
            Self::BO => ("BO", "Kiescollege Bonaire", "Kieskolleezje Bonêre"),
            Self::SE => (
                "SE",
                "Kiescollege Sint Eustatius",
                "Kieskolleezje Sint Eustaasjus",
            ),
            Self::SA => ("SA", "Kiescollege Saba", "Kieskolleezje Saba"),
            Self::KN => (
                "KN",
                "Kiescollege Niet-Ingezetenen",
                "Kieskolleezje Net-Ynwenners",
            ),
            Self::WsNoorderzijlvest => ("WS-NZV", "Noorderzijlvest", "Noorderzijlvest"),
            Self::WsFryslan => ("WS-FRY", "Fryslân", "Fryslân"),
            Self::WsHunzeEnAas => ("WS-HEA", "Hunze en Aa's", "Hunze en Aa's"),
            Self::WsDrentsOverijsselseDelta => (
                "WS-DOD",
                "Drents Overijsselse Delta",
                "Drents Overijsselse Delta",
            ),
            Self::WsVechtstromen => ("WS-VST", "Vechtstromen", "Vechtstromen"),
            Self::WsValleiEnVeluwe => ("WS-VEV", "Vallei en Veluwe", "Vallei en Veluwe"),
            Self::WsRijnEnIJssel => ("WS-REI", "Rijn en IJssel", "Rijn en IJssel"),
            Self::WsDeStichtseRijnlanden => {
                ("WS-SRL", "De Stichtse Rijnlanden", "De Stichtse Rijnlanden")
            }
            Self::WsAmstelGooiEnVecht => {
                ("WS-AGV", "Amstel, Gooi en Vecht", "Amstel, Gooi en Vecht")
            }
            Self::WsHollandsNoorderkwartier => (
                "WS-HNK",
                "Hollands Noorderkwartier",
                "Hollands Noorderkwartier",
            ),
            Self::WsRijnland => ("WS-RNL", "Rijnland", "Rijnland"),
            Self::WsDelfland => ("WS-DFL", "Delfland", "Delfland"),
            Self::WsSchielandEnDeKrimpenerwaard => (
                "WS-SKW",
                "Schieland en de Krimpenerwaard",
                "Schieland en de Krimpenerwaard",
            ),
            Self::WsRivierenland => ("WS-RVL", "Rivierenland", "Rivierenland"),
            Self::WsHollandseDelta => ("WS-HDT", "Hollandse Delta", "Hollandse Delta"),
            Self::WsScheldestromen => ("WS-SDS", "Scheldestromen", "Scheldestromen"),
            Self::WsBrabantseDelta => ("WS-BDT", "Brabantse Delta", "Brabantse Delta"),
            Self::WsDeDommel => ("WS-DDM", "De Dommel", "De Dommel"),
            Self::WsAaEnMaas => ("WS-AEM", "Aa en Maas", "Aa en Maas"),
            Self::WsLimburg => ("WS-LMB", "Limburg", "Limburg"),
            Self::WsZuiderzeeland => ("WS-ZZL", "Zuiderzeeland", "Zuiderzeeland"),
        }
    }

    pub fn code(&self) -> &'static str {
        self.data().0
    }

    pub fn title(&self, locale: AnyLocale) -> &'static str {
        let (_code, default, frisian) = self.data();
        match locale {
            AnyLocale::Fry => frisian,
            AnyLocale::En => self.english_title().unwrap_or(default),
            AnyLocale::Nl => default,
        }
    }

    /// English overrides for districts that differ from the default (Dutch) title.
    fn english_title(&self) -> Option<&'static str> {
        match self {
            Self::NH => Some("North Holland"),
            Self::ZH => Some("South Holland"),
            Self::PsDenHaag => Some("The Hague"),
            Self::NB => Some("North Brabant"),
            Self::BO => Some("Electoral College Bonaire"),
            Self::SE => Some("Electoral College Sint Eustatius"),
            Self::SA => Some("Electoral College Saba"),
            Self::KN => Some("Electoral College Non-Residents"),
            _ => None,
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

        assert_eq!(ElectoralDistrict::WsHunzeEnAas.code(), "WS-HEA");
        assert_eq!(
            ElectoralDistrict::WsHunzeEnAas.title(AnyLocale::Nl),
            "Hunze en Aa's"
        );
        assert_eq!(ElectoralDistrict::WsAmstelGooiEnVecht.code(), "WS-AGV");
        assert_eq!(
            ElectoralDistrict::WsAmstelGooiEnVecht.title(AnyLocale::Nl),
            "Amstel, Gooi en Vecht"
        );

        assert_eq!(ElectoralDistrict::FR.title(AnyLocale::Nl), "Friesland");
        assert_eq!(ElectoralDistrict::FR.title(AnyLocale::Fry), "Fryslân");
        assert_eq!(ElectoralDistrict::WsFryslan.title(AnyLocale::Nl), "Fryslân");
        assert_eq!(
            ElectoralDistrict::PsLeeuwarden.title(AnyLocale::Nl),
            "Leeuwarden"
        );
        assert_eq!(
            ElectoralDistrict::PsLeeuwarden.title(AnyLocale::Fry),
            "Ljouwert"
        );
    }

    #[test]
    fn electoral_districts_include_expected_code() {
        let districts = ElectoralDistrict::ek27();
        assert!(districts.contains(&ElectoralDistrict::UT));
        assert_eq!(districts.len(), 16);
    }

    #[test]
    fn similar_districts_have_different_codes() {
        assert_eq!(
            ElectoralDistrict::FR.title(AnyLocale::Fry),
            ElectoralDistrict::WsFryslan.title(AnyLocale::Fry)
        );
        assert_ne!(
            ElectoralDistrict::FR.code(),
            ElectoralDistrict::WsFryslan.code()
        );

        assert_eq!(
            ElectoralDistrict::GR.title(AnyLocale::Nl),
            ElectoralDistrict::PsGroningen.title(AnyLocale::Nl)
        );
        assert_ne!(
            ElectoralDistrict::GR.code(),
            ElectoralDistrict::PsGroningen.code()
        );

        assert_eq!(
            ElectoralDistrict::UT.title(AnyLocale::Nl),
            ElectoralDistrict::PsUtrecht.title(AnyLocale::Nl)
        );
        assert_ne!(
            ElectoralDistrict::UT.code(),
            ElectoralDistrict::PsUtrecht.code()
        );

        assert_eq!(
            ElectoralDistrict::LI.title(AnyLocale::Nl),
            ElectoralDistrict::WsLimburg.title(AnyLocale::Nl)
        );
        assert_ne!(
            ElectoralDistrict::LI.code(),
            ElectoralDistrict::WsLimburg.code()
        );
    }
}
