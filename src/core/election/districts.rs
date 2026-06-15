use crate::core::AnyLocale;

super::define_districts! {
    GR("1", "GR", "Groningen", fry: "Grinslân"),
    PsGroningen("1", "GRQ", "Groningen"),
    FR("2", "FR", "Fryslân", fry: "Fryslân"),
    PsLeeuwarden("2", "LWR", "Leeuwarden", fry: "Ljouwert"),
    DR("3", "DR", "Drenthe", fry: "Drinte"),
    PsAssen("3", "ASS", "Assen"),
    OV("4", "OV", "Overijssel", fry: "Oerisel"),
    PsZwolle("4", "ZWO", "Zwolle"),
    FL("5", "FL", "Flevoland", fry: "Flevolân"),
    PsLelystad("5", "LEY", "Lelystad"),
    GE("6", "GE", "Gelderland", fry: "Gelderlân"),
    PsNijmegen("6", "NIJ", "Nijmegen"),
    PsArnhem("7", "ARN", "Arnhem"),
    UT("7", "UT", "Utrecht", fry: "Utert"),
    PsUtrecht("8", "UTC", "Utrecht"),
    NH("8", "NH", "Noord-Holland", fry: "Noard-Hollân", en: "North Holland"),
    PsAmsterdam("9", "AMS", "Amsterdam"),
    PsHaarlem("10", "HAA", "Haarlem"),
    PsDenHelder("11", "DHR", "Den Helder"),
    ZH("9", "ZH", "Zuid-Holland", fry: "Súd-Hollân", en: "South Holland"),
    PsDenHaag("12", "HAG", "'s-Gravenhage", en: "The Hague"),
    PsRotterdam("13", "RTM", "Rotterdam"),
    PsDordrecht("14", "DOR", "Dordrecht"),
    PsLeiden("15", "LID", "Leiden"),
    ZE("10", "ZE", "Zeeland", fry: "Seelân"),
    PsMiddelburg("16", "MDL", "Middelburg"),
    NB("11", "NB", "Noord-Brabant", fry: "Noard-Brabân", en: "North Brabant"),
    PsTilburg("17", "TLB", "Tilburg"),
    PsDenBosch("18", "HTB", "'s-Hertogenbosch"),
    LI("12", "LI", "Limburg", fry: "Limboarch"),
    PsMaastricht("19", "MST", "Maastricht"),
    PsVenlo("20", "VEN", "Venlo"),
    BO("13", "BO", "Bonaire", fry: "Bonêre", en: "Bonaire"),
    SE("14", "SE", "Sint Eustatius", fry: "Sint Eustaasjus", en: "Sint Eustatius"),
    SA("15", "SA", "Saba", fry: "Saba", en: "Saba"),
    KN("16", "KN", "Buitenland", fry: "Bûtenlân", en: "Abroad"),
    WsNoorderzijlvest("1", "WS-NZV", "Noorderzijlvest"),
    WsFryslan("2", "WS-FRY", "Fryslân"),
    WsHunzeEnAas("3", "WS-HEA", "Hunze en Aa's"),
    WsDrentsOverijsselseDelta("24", "WS-DOD", "Drents Overijsselse Delta"),
    WsVechtstromen("5", "WS-VST", "Vechtstromen"),
    WsValleiEnVeluwe("7", "WS-VEV", "Vallei en Veluwe"),
    WsRijnEnIJssel("8", "WS-REI", "Rijn en IJssel"),
    WsDeStichtseRijnlanden("9", "WS-SRL", "De Stichtse Rijnlanden"),
    WsAmstelGooiEnVecht("10", "WS-AGV", "Amstel, Gooi en Vecht"),
    WsHollandsNoorderkwartier("11", "WS-HNK", "Hollands Noorderkwartier"),
    WsRijnland("12", "WS-RNL", "Rijnland"),
    WsDelfland("13", "WS-DFL", "Delfland"),
    WsSchielandEnDeKrimpenerwaard("14", "WS-SKW", "Schieland en de Krimpenerwaard"),
    WsRivierenland("15", "WS-RVL", "Rivierenland"),
    WsHollandseDelta("16", "WS-HDT", "Hollandse Delta"),
    WsScheldestromen("17", "WS-SDS", "Scheldestromen"),
    WsBrabantseDelta("18", "WS-BDT", "Brabantse Delta"),
    WsDeDommel("19", "WS-DDM", "De Dommel"),
    WsAaEnMaas("20", "WS-AEM", "Aa en Maas"),
    WsLimburg("25", "WS-LMB", "Limburg"),
    WsZuiderzeeland("23", "WS-ZZL", "Zuiderzeeland"),
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn district_title_and_code_match() {
        assert_eq!(ElectoralDistrict::UT.code(), "UT");
        assert_eq!(ElectoralDistrict::UT.region_number(), "7");
        assert_eq!(ElectoralDistrict::UT.title(AnyLocale::Nl), "Utrecht");
        assert_eq!(ElectoralDistrict::UT.title(AnyLocale::Fry), "Utert");

        assert_eq!(ElectoralDistrict::PsArnhem.code(), "ARN");
        assert_eq!(ElectoralDistrict::PsArnhem.region_number(), "7");
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
        assert_eq!(ElectoralDistrict::WsHunzeEnAas.region_number(), "3");
        assert_eq!(
            ElectoralDistrict::WsHunzeEnAas.title(AnyLocale::Nl),
            "Hunze en Aa's"
        );
        assert_eq!(ElectoralDistrict::WsAmstelGooiEnVecht.code(), "WS-AGV");
        assert_eq!(ElectoralDistrict::WsAmstelGooiEnVecht.region_number(), "10");
        assert_eq!(
            ElectoralDistrict::WsAmstelGooiEnVecht.title(AnyLocale::Nl),
            "Amstel, Gooi en Vecht"
        );

        assert_eq!(ElectoralDistrict::FR.title(AnyLocale::Nl), "Fryslân");
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

        assert_eq!(ElectoralDistrict::KN.region_number(), "16");
        assert_eq!(
            ElectoralDistrict::WsDrentsOverijsselseDelta.region_number(),
            "24"
        );
        assert_eq!(ElectoralDistrict::WsLimburg.region_number(), "25");
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
