use serde::{Deserialize, Serialize};

use crate::{ElectoralDistrict as District, core::AnyLocale};

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
            Self::GR => District::GR,
            Self::FR => District::FR,
            Self::DR => District::DR,
            Self::OV => District::OV,
            Self::GE => District::GE,
            Self::FL => District::FL,
            Self::UT => District::UT,
            Self::NH => District::NH,
            Self::ZH => District::ZH,
            Self::ZE => District::ZE,
            Self::NB => District::NB,
            Self::LI => District::LI,
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
            Self::Noorderzijlvest => District::WsNoorderzijlvest,
            Self::Fryslan => District::WsFryslan,
            Self::HunzeEnAas => District::WsHunzeEnAas,
            Self::DrentsOverijsselseDelta => District::WsDrentsOverijsselseDelta,
            Self::Vechtstromen => District::WsVechtstromen,
            Self::ValleiEnVeluwe => District::WsValleiEnVeluwe,
            Self::RijnEnIJssel => District::WsRijnEnIJssel,
            Self::DeStichtseRijnlanden => District::WsDeStichtseRijnlanden,
            Self::AmstelGooiEnVecht => District::WsAmstelGooiEnVecht,
            Self::HollandsNoorderkwartier => District::WsHollandsNoorderkwartier,
            Self::Rijnland => District::WsRijnland,
            Self::Delfland => District::WsDelfland,
            Self::SchielandEnDeKrimpenerwaard => District::WsSchielandEnDeKrimpenerwaard,
            Self::Rivierenland => District::WsRivierenland,
            Self::HollandseDelta => District::WsHollandseDelta,
            Self::Scheldestromen => District::WsScheldestromen,
            Self::BrabantseDelta => District::WsBrabantseDelta,
            Self::DeDommel => District::WsDeDommel,
            Self::AaEnMaas => District::WsAaEnMaas,
            Self::Limburg => District::WsLimburg,
            Self::Zuiderzeeland => District::WsZuiderzeeland,
        };
        district.title(locale)
    }
}
