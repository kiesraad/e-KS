use serde::{Deserialize, Serialize};

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
