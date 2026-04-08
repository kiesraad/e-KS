use chrono::NaiveDate;

use crate::{
    ElectoralDistrict,
    core::{
        AnyLocale, ElectionType,
        election::{Province, WaterCouncil},
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
        nomination_day_date: NaiveDate::from_ymd_opt(2027, 4, 20).unwrap(),
        eligible_date_of_birth: NaiveDate::from_ymd_opt(2014, 4, 20).unwrap(), // TODO: determine definitive date
        electoral_districts: ElectoralDistrict::ek27(),
    },

    PS27(province: Province) {
        election_type: ElectionType::Ps(*province),
        titles: {
            nl: "Provinciale Statenverkiezingen 2027",
            fry: "Provinsjale Steateferkiezings 2027",
            en: "Elections of the Provincial Council 2027",
        },
        nomination_day_date: NaiveDate::from_ymd_opt(2027, 2, 1).unwrap(),
        eligible_date_of_birth: NaiveDate::from_ymd_opt(2014, 2, 1).unwrap(), // TODO: determine definitive date
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
    },

    WS27(water_council: WaterCouncil) {
        election_type: ElectionType::Ws(*water_council),
        titles: {
            nl: "Waterschapsverkiezingen 2027",
            fry: "Wetterskipsferkiezings 2027",
            en: "Elections of the Water Authority 2027",
        },
        nomination_day_date: NaiveDate::from_ymd_opt(2027, 2, 1).unwrap(),
        eligible_date_of_birth: NaiveDate::from_ymd_opt(2014, 2, 1).unwrap(), // TODO: determine definitive date
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
    }
}

impl ElectionConfig {
    pub fn get_max_candidates(&self, long_list_allowed: bool) -> usize {
        if long_list_allowed { 80 } else { 50 }
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
}
