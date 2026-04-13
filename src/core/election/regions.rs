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
    pub const ALL: &[Province] = &[
        Province::GR,
        Province::FR,
        Province::DR,
        Province::OV,
        Province::FL,
        Province::GE,
        Province::UT,
        Province::NH,
        Province::ZH,
        Province::ZE,
        Province::NB,
        Province::LI,
    ];

    pub fn code(&self) -> &'static str {
        ElectoralDistrict::from(*self).code()
    }

    pub fn title(&self, locale: AnyLocale) -> &'static str {
        ElectoralDistrict::from(*self).title(locale)
    }

    pub fn from_code(code: &str) -> Option<Self> {
        Self::ALL.iter().find(|x| x.code() == code).copied()
    }
}

impl From<Province> for ElectoralDistrict {
    fn from(province: Province) -> Self {
        match province {
            Province::GR => Self::GR,
            Province::FR => Self::FR,
            Province::DR => Self::DR,
            Province::OV => Self::OV,
            Province::FL => Self::FL,
            Province::GE => Self::GE,
            Province::UT => Self::UT,
            Province::NH => Self::NH,
            Province::ZH => Self::ZH,
            Province::ZE => Self::ZE,
            Province::NB => Self::NB,
            Province::LI => Self::LI,
        }
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
    pub const ALL: &[WaterCouncil] = &[
        WaterCouncil::Noorderzijlvest,
        WaterCouncil::Fryslan,
        WaterCouncil::HunzeEnAas,
        WaterCouncil::DrentsOverijsselseDelta,
        WaterCouncil::Vechtstromen,
        WaterCouncil::ValleiEnVeluwe,
        WaterCouncil::RijnEnIJssel,
        WaterCouncil::DeStichtseRijnlanden,
        WaterCouncil::AmstelGooiEnVecht,
        WaterCouncil::HollandsNoorderkwartier,
        WaterCouncil::Rijnland,
        WaterCouncil::Delfland,
        WaterCouncil::SchielandEnDeKrimpenerwaard,
        WaterCouncil::Rivierenland,
        WaterCouncil::HollandseDelta,
        WaterCouncil::Scheldestromen,
        WaterCouncil::BrabantseDelta,
        WaterCouncil::DeDommel,
        WaterCouncil::AaEnMaas,
        WaterCouncil::Limburg,
        WaterCouncil::Zuiderzeeland,
    ];

    pub fn code(&self) -> &'static str {
        ElectoralDistrict::from(*self).code()
    }

    pub fn title(&self, locale: AnyLocale) -> &'static str {
        ElectoralDistrict::from(*self).title(locale)
    }

    pub fn from_code(code: &str) -> Option<Self> {
        Self::ALL.iter().find(|x| x.code() == code).copied()
    }
}

impl From<WaterCouncil> for ElectoralDistrict {
    fn from(wc: WaterCouncil) -> Self {
        match wc {
            WaterCouncil::Noorderzijlvest => Self::WsNoorderzijlvest,
            WaterCouncil::Fryslan => Self::WsFryslan,
            WaterCouncil::HunzeEnAas => Self::WsHunzeEnAas,
            WaterCouncil::DrentsOverijsselseDelta => Self::WsDrentsOverijsselseDelta,
            WaterCouncil::Vechtstromen => Self::WsVechtstromen,
            WaterCouncil::ValleiEnVeluwe => Self::WsValleiEnVeluwe,
            WaterCouncil::RijnEnIJssel => Self::WsRijnEnIJssel,
            WaterCouncil::DeStichtseRijnlanden => Self::WsDeStichtseRijnlanden,
            WaterCouncil::AmstelGooiEnVecht => Self::WsAmstelGooiEnVecht,
            WaterCouncil::HollandsNoorderkwartier => Self::WsHollandsNoorderkwartier,
            WaterCouncil::Rijnland => Self::WsRijnland,
            WaterCouncil::Delfland => Self::WsDelfland,
            WaterCouncil::SchielandEnDeKrimpenerwaard => Self::WsSchielandEnDeKrimpenerwaard,
            WaterCouncil::Rivierenland => Self::WsRivierenland,
            WaterCouncil::HollandseDelta => Self::WsHollandseDelta,
            WaterCouncil::Scheldestromen => Self::WsScheldestromen,
            WaterCouncil::BrabantseDelta => Self::WsBrabantseDelta,
            WaterCouncil::DeDommel => Self::WsDeDommel,
            WaterCouncil::AaEnMaas => Self::WsAaEnMaas,
            WaterCouncil::Limburg => Self::WsLimburg,
            WaterCouncil::Zuiderzeeland => Self::WsZuiderzeeland,
        }
    }
}
