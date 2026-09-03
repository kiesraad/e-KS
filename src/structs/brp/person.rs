use serde::Deserialize;

/// Nationality code of "Nederlandse" in the national `Nationaliteiten` table.
pub const DUTCH_NATIONALITY_CODE: &str = "0001";

/// A person as returned by the BRP, in the shapes the API actually uses.
///
/// Values are deliberately left unparsed here. The check has to tell "the BRP
/// holds a different value" apart from "the BRP holds a value this application
/// cannot interpret", and that distinction only exists while the raw value is
/// still available. Parsing happens where the comparison happens, in
/// [`super::client`].
#[derive(Debug, Clone, Default, Deserialize)]
pub struct BrpPerson {
    #[serde(rename = "burgerservicenummer")]
    pub bsn: Option<String>,
    #[serde(rename = "naam")]
    pub name: Option<BrpName>,
    #[serde(rename = "geslacht")]
    pub gender: Option<BrpValue>,
    #[serde(rename = "geboorte")]
    pub birth: Option<BrpBirth>,
    #[serde(rename = "overlijden")]
    pub death: Option<BrpDeath>,
    #[serde(rename = "nationaliteiten", default)]
    pub nationalities: Vec<BrpNationality>,
    #[serde(rename = "uitsluitingKiesrecht")]
    pub suffrage_exclusion: Option<BrpSuffrageExclusion>,
    #[serde(rename = "verblijfplaats")]
    pub residence: Option<BrpResidence>,
}

/// An entry from one of the BRP's national code tables ("waardetabel").
#[derive(Debug, Clone, Default, Deserialize)]
pub struct BrpValue {
    #[serde(rename = "code")]
    pub code: Option<String>,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct BrpName {
    #[serde(rename = "geslachtsnaam")]
    pub last_name: Option<String>,
    #[serde(rename = "voorvoegsel")]
    pub last_name_prefix: Option<String>,
    #[serde(rename = "voorletters")]
    pub initials: Option<String>,
}

/// A BRP date. Partial dates (a year without a month, for instance) are
/// returned without a `datum`, so an absent `datum` means "not a full date".
#[derive(Debug, Clone, Default, Deserialize)]
pub struct BrpDate {
    #[serde(rename = "datum")]
    pub date: Option<String>,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct BrpBirth {
    #[serde(rename = "datum")]
    pub date: Option<BrpDate>,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct BrpDeath {
    #[serde(rename = "datum")]
    pub date: Option<BrpDate>,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct BrpNationality {
    #[serde(rename = "nationaliteit")]
    pub nationality: Option<BrpValue>,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct BrpSuffrageExclusion {
    #[serde(rename = "uitgeslotenVanKiesrecht")]
    pub excluded: Option<bool>,
}

/// `verblijfplaats` is polymorphic on its `type`. Only `Adres` carries a
/// `woonplaats`; the other three shapes are distinguished here rather than
/// lumped together, because "lives abroad", "has a briefadres" and "residence
/// unknown" are three different things for the committee to look at.
#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "type")]
pub enum BrpResidence {
    #[serde(rename = "Adres")]
    Address {
        #[serde(rename = "verblijfadres")]
        address: Option<BrpAddress>,
    },
    #[serde(rename = "Locatie")]
    Location,
    #[serde(rename = "VerblijfplaatsBuitenland")]
    Abroad,
    #[serde(rename = "VerblijfplaatsOnbekend")]
    Unknown,
    /// A `type` this application does not know; treated like an unknown
    /// residence rather than silently skipped.
    #[serde(other)]
    Other,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct BrpAddress {
    #[serde(rename = "woonplaats")]
    pub place_of_residence: Option<String>,
}

impl BrpPerson {
    /// The full date of birth, if the BRP holds one.
    pub fn date_of_birth(&self) -> Option<&str> {
        self.birth.as_ref()?.date.as_ref()?.date.as_deref()
    }

    /// The date of death, if the BRP records one.
    pub fn date_of_death(&self) -> Option<&str> {
        self.death.as_ref()?.date.as_ref()?.date.as_deref()
    }

    /// The `geslacht` code (`M`, `V` or `O`).
    pub fn gender_code(&self) -> Option<&str> {
        self.gender.as_ref()?.code.as_deref()
    }

    /// Whether the BRP records Dutch nationality. `BehandeldAlsNederlander`
    /// does not count: article 56 of the Grondwet requires being a Dutch
    /// national, and that variant carries no `nationaliteit` code.
    pub fn is_dutch(&self) -> bool {
        self.nationalities.iter().any(|entry| {
            entry
                .nationality
                .as_ref()
                .and_then(|value| value.code.as_deref())
                == Some(DUTCH_NATIONALITY_CODE)
        })
    }

    /// Whether the BRP records this person as excluded from the right to vote.
    pub fn is_excluded_from_suffrage(&self) -> bool {
        self.suffrage_exclusion
            .as_ref()
            .and_then(|exclusion| exclusion.excluded)
            .unwrap_or(false)
    }
}
