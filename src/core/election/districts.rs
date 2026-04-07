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

    pub fn title(&self, locale: AnyLocale) -> &'static str {
        match (self, locale) {
            (Self::GR, AnyLocale::Nl | AnyLocale::En) => "Groningen",
            (Self::GR, AnyLocale::Fry) => "Grinslân",
            (Self::PsGroningen, _) => "Groningen",
            (Self::FR, AnyLocale::Nl | AnyLocale::En) => "Friesland",
            (Self::FR, AnyLocale::Fry) => "Fryslân",
            (Self::PsLeeuwarden, AnyLocale::Nl | AnyLocale::En) => "Leeuwarden",
            (Self::PsLeeuwarden, AnyLocale::Fry) => "Ljouwert",
            (Self::DR, AnyLocale::Nl | AnyLocale::En) => "Drenthe",
            (Self::DR, AnyLocale::Fry) => "Drinte",
            (Self::PsAssen, _) => "Assen",
            (Self::OV, AnyLocale::Nl | AnyLocale::En) => "Overijssel",
            (Self::OV, AnyLocale::Fry) => "Oerisel",
            (Self::PsZwolle, _) => "Zwolle",
            (Self::FL, AnyLocale::Nl | AnyLocale::En) => "Flevoland",
            (Self::FL, AnyLocale::Fry) => "Flevolân",
            (Self::PsLelystad, _) => "Lelystad",
            (Self::GE, AnyLocale::Nl | AnyLocale::En) => "Gelderland",
            (Self::GE, AnyLocale::Fry) => "Gelderlân",
            (Self::PsNijmegen, _) => "Nijmegen",
            (Self::PsArnhem, _) => "Arnhem",
            (Self::UT, AnyLocale::Nl | AnyLocale::En) => "Utrecht",
            (Self::UT, AnyLocale::Fry) => "Utert",
            (Self::PsUtrecht, _) => "Utrecht",
            (Self::NH, AnyLocale::Nl) => "Noord-Holland",
            (Self::NH, AnyLocale::En) => "North Holland",
            (Self::NH, AnyLocale::Fry) => "Noard-Hollân",
            (Self::PsAmsterdam, _) => "Amsterdam",
            (Self::PsHaarlem, _) => "Haarlem",
            (Self::PsDenHelder, _) => "Den Helder",
            (Self::ZH, AnyLocale::Nl) => "Zuid-Holland",
            (Self::ZH, AnyLocale::En) => "South Holland",
            (Self::ZH, AnyLocale::Fry) => "Súd-Hollân",
            (Self::PsDenHaag, AnyLocale::Nl | AnyLocale::Fry) => "'s-Gravenhage",
            (Self::PsDenHaag, AnyLocale::En) => "The Hague",
            (Self::PsRotterdam, _) => "Rotterdam",
            (Self::PsDordrecht, _) => "Dordrecht",
            (Self::PsLeiden, _) => "Leiden",
            (Self::ZE, AnyLocale::Nl | AnyLocale::En) => "Zeeland",
            (Self::ZE, AnyLocale::Fry) => "Seelân",
            (Self::PsMiddelburg, _) => "Middelburg",
            (Self::NB, AnyLocale::Nl) => "Noord-Brabant",
            (Self::NB, AnyLocale::En) => "North Brabant",
            (Self::NB, AnyLocale::Fry) => "Noard-Brabân",
            (Self::PsTilburg, _) => "Tilburg",
            (Self::PsDenBosch, _) => "'s-Hertogenbosch",
            (Self::LI, AnyLocale::Nl | AnyLocale::En) => "Limburg",
            (Self::LI, AnyLocale::Fry) => "Limboarch",
            (Self::PsMaastricht, _) => "Maastricht",
            (Self::PsVenlo, _) => "Venlo",
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
            (Self::WsNoorderzijlvest, _) => "Noorderzijlvest",
            (Self::WsFryslan, _) => "Fryslân",
            (Self::WsHunzeEnAas, _) => "Hunze en Aa's",
            (Self::WsDrentsOverijsselseDelta, _) => "Drents Overijsselse Delta",
            (Self::WsVechtstromen, _) => "Vechtstromen",
            (Self::WsValleiEnVeluwe, _) => "Vallei en Veluwe",
            (Self::WsRijnEnIJssel, _) => "Rijn en IJssel",
            (Self::WsDeStichtseRijnlanden, _) => "De Stichtse Rijnlanden",
            (Self::WsAmstelGooiEnVecht, _) => "Amstel, Gooi en Vecht",
            (Self::WsHollandsNoorderkwartier, _) => "Hollands Noorderkwartier",
            (Self::WsRijnland, _) => "Rijnland",
            (Self::WsDelfland, _) => "Delfland",
            (Self::WsSchielandEnDeKrimpenerwaard, _) => "Schieland en de Krimpenerwaard",
            (Self::WsRivierenland, _) => "Rivierenland",
            (Self::WsHollandseDelta, _) => "Hollandse Delta",
            (Self::WsScheldestromen, _) => "Scheldestromen",
            (Self::WsBrabantseDelta, _) => "Brabantse Delta",
            (Self::WsDeDommel, _) => "De Dommel",
            (Self::WsAaEnMaas, _) => "Aa en Maas",
            (Self::WsLimburg, _) => "Limburg",
            (Self::WsZuiderzeeland, _) => "Zuiderzeeland",
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
            Self::WsNoorderzijlvest => "WS-NZV",
            Self::WsFryslan => "WS-FRY",
            Self::WsHunzeEnAas => "WS-HEA",
            Self::WsDrentsOverijsselseDelta => "WS-DOD",
            Self::WsVechtstromen => "WS-VST",
            Self::WsValleiEnVeluwe => "WS-VEV",
            Self::WsRijnEnIJssel => "WS-REI",
            Self::WsDeStichtseRijnlanden => "WS-SRL",
            Self::WsAmstelGooiEnVecht => "WS-AGV",
            Self::WsHollandsNoorderkwartier => "WS-HNK",
            Self::WsRijnland => "WS-RNL",
            Self::WsDelfland => "WS-DFL",
            Self::WsSchielandEnDeKrimpenerwaard => "WS-SKW",
            Self::WsRivierenland => "WS-RVL",
            Self::WsHollandseDelta => "WS-HDT",
            Self::WsScheldestromen => "WS-SDS",
            Self::WsBrabantseDelta => "WS-BDT",
            Self::WsDeDommel => "WS-DDM",
            Self::WsAaEnMaas => "WS-AEM",
            Self::WsLimburg => "WS-LMB",
            Self::WsZuiderzeeland => "WS-ZZL",
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
