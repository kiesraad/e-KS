use chrono::NaiveDate;

use crate::{
    ElectoralDistrict,
    core::{AnyLocale, ElectionType, election::Province},
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
        electoral_districts: ElectoralDistrict::ek27(),
    },

    PS27(province: Province) {
        election_type: ElectionType::Ps,
        titles: {
            nl: "Provinciale Statenverkiezingen 2027",
            fry: "Provinsjale Steateferkiezings 2027",
            en: "Elections of the provincial council 2027",
        },
        nomination_day_date: NaiveDate::from_ymd_opt(2027, 1, 1).unwrap(),
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
    }
}
