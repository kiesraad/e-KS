use serde::{Deserialize, Serialize};

use crate::{ElectoralDistrict, core::AnyLocale};

/// Regions for the elections of the provincial councils
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

impl Province {
    pub fn title(&self, locale: AnyLocale) -> &'static str {
        let district = match self {
            Self::GR => ElectoralDistrict::GR,
            Self::FR => ElectoralDistrict::FR,
            Self::DR => ElectoralDistrict::DR,
            Self::OV => ElectoralDistrict::OV,
            Self::GE => ElectoralDistrict::GE,
            Self::FL => ElectoralDistrict::FL,
            Self::UT => ElectoralDistrict::UT,
            Self::NH => ElectoralDistrict::NH,
            Self::ZH => ElectoralDistrict::ZH,
            Self::ZE => ElectoralDistrict::ZE,
            Self::NB => ElectoralDistrict::NB,
            Self::LI => ElectoralDistrict::LI,
        };
        district.title(locale)
    }
}

/// Regions for the elections of the water authorities
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum WaterCouncil {
    Noorderzijlvest,
    Fryslan,
    HunzeEnAas,
    DrentsOverijsselseDelta,
    Vechtstromen,
    ValleiEnVeluwe,
    RijnEnIJssel,
    DeStichtseRijnlanden,
    AmstelGooiEnVecht,
    HollandsNoorderkwartier,
    Rijnland,
    Delfland,
    SchielandEnDeKrimpenerwaard,
    Rivierenland,
    HollandseDelta,
    Scheldestromen,
    BrabantseDelta,
    DeDommel,
    AaEnMaas,
    Limburg,
    Zuiderzeeland,
}

impl WaterCouncil {
    pub fn title(&self, locale: AnyLocale) -> &'static str {
        let district = match self {
            Self::Noorderzijlvest => ElectoralDistrict::WsNoorderzijlvest,
            Self::Fryslan => ElectoralDistrict::WsFryslan,
            Self::HunzeEnAas => ElectoralDistrict::WsHunzeEnAas,
            Self::DrentsOverijsselseDelta => ElectoralDistrict::WsDrentsOverijsselseDelta,
            Self::Vechtstromen => ElectoralDistrict::WsVechtstromen,
            Self::ValleiEnVeluwe => ElectoralDistrict::WsValleiEnVeluwe,
            Self::RijnEnIJssel => ElectoralDistrict::WsRijnEnIJssel,
            Self::DeStichtseRijnlanden => ElectoralDistrict::WsDeStichtseRijnlanden,
            Self::AmstelGooiEnVecht => ElectoralDistrict::WsAmstelGooiEnVecht,
            Self::HollandsNoorderkwartier => ElectoralDistrict::WsHollandsNoorderkwartier,
            Self::Rijnland => ElectoralDistrict::WsRijnland,
            Self::Delfland => ElectoralDistrict::WsDelfland,
            Self::SchielandEnDeKrimpenerwaard => ElectoralDistrict::WsSchielandEnDeKrimpenerwaard,
            Self::Rivierenland => ElectoralDistrict::WsRivierenland,
            Self::HollandseDelta => ElectoralDistrict::WsHollandseDelta,
            Self::Scheldestromen => ElectoralDistrict::WsScheldestromen,
            Self::BrabantseDelta => ElectoralDistrict::WsBrabantseDelta,
            Self::DeDommel => ElectoralDistrict::WsDeDommel,
            Self::AaEnMaas => ElectoralDistrict::WsAaEnMaas,
            Self::Limburg => ElectoralDistrict::WsLimburg,
            Self::Zuiderzeeland => ElectoralDistrict::WsZuiderzeeland,
        };
        district.title(locale)
    }
}
