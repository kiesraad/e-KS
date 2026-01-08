use strum::{Display, EnumString};

use crate::Locale;

#[derive(Debug, Display, EnumString, PartialEq, Eq, Clone, Copy)]
pub enum Country {
    NL,
    BE,
    DE,
    FR,
    UK,
    US,
}

impl Country {
    pub fn label(&self, locale: &Locale) -> &'static str {
        match locale {
            Locale::En => match self {
                Country::NL => "Netherlands",
                Country::BE => "Belgium",
                Country::DE => "Germany",
                Country::FR => "France",
                Country::UK => "United Kingdom",
                Country::US => "United States",
            },
            Locale::Nl => match self {
                Country::NL => "Nederland",
                Country::BE => "België",
                Country::DE => "Duitsland",
                Country::FR => "Frankrijk",
                Country::UK => "Verenigd Koninkrijk",
                Country::US => "Verenigde Staten",
            },
        }
    }
}
