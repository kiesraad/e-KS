use chrono::NaiveDate;

use crate::core::{AnyLocale, ElectionType, ElectoralDistrict};

/// Active election configuration and ruleset for the application.
#[derive(Default, Debug, Clone, Copy)]
pub enum ElectionConfig {
    #[default]
    EK2027,
}

impl ElectionConfig {
    pub fn election_type(&self) -> ElectionType {
        match self {
            Self::EK2027 => ElectionType::Ek,
        }
    }

    pub fn title(&self, locale: AnyLocale) -> &'static str {
        match self {
            Self::EK2027 => match locale {
                AnyLocale::En => "Election of the Senate of the States General 2027",
                AnyLocale::Fry => "Earste Keamerferkiezings fan de Steaten-Generaal 2027",
                AnyLocale::Nl => "Eerste Kamerverkiezing der Staten-Generaal 2027",
            },
        }
    }

    pub fn short_title(&self, locale: AnyLocale) -> &'static str {
        match self {
            Self::EK2027 => match locale {
                AnyLocale::En => "Election of the Senate 2027",
                AnyLocale::Fry => "Earste Keamer 2027",
                AnyLocale::Nl => "Eerste Kamer 2027",
            },
        }
    }

    pub fn nomination_day_date(&self) -> NaiveDate {
        match self {
            ElectionConfig::EK2027 => NaiveDate::from_ymd_opt(2027, 4, 20).unwrap(),
        }
    }

    pub fn get_max_candidates(&self, long_list_allowed: bool) -> usize {
        if long_list_allowed { 80 } else { 50 }
    }

    pub fn electoral_districts(&self) -> &'static [ElectoralDistrict] {
        match self {
            Self::EK2027 => ElectoralDistrict::ek2027(),
        }
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
        assert!(ElectionConfig::EK2027.title(AnyLocale::Nl).len() > 20);
        assert!(ElectionConfig::EK2027.short_title(AnyLocale::Nl).len() > 10);

        let election_type = ElectionConfig::EK2027.election_type();
        assert!(election_type.title(Locale::Nl).len() > 20);
    }

    #[test]
    fn election_config_exposes_districts() {
        let districts = ElectionConfig::EK2027.electoral_districts();
        assert!(districts.contains(&ElectoralDistrict::NH));
    }
}
