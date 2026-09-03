use serde::Serialize;

/// A field that can be requested from the BRP `personen` endpoint, named by
/// the dotted path the API expects in the `fields` array.
///
/// The variants that are actually requested are listed in
/// [`super::client::CANDIDATE_FIELDS`]; the others document paths this
/// application may need later.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum BrpField {
    // Personen
    #[serde(rename = "burgerservicenummer")]
    Bsn,
    #[serde(rename = "naam.voornamen")]
    FirstNames,
    #[serde(rename = "naam.voorletters")]
    Initials,
    #[serde(rename = "naam.adellijkeTitelPredicaat")]
    TitleOfNobility,
    #[serde(rename = "naam.voorvoegsel")]
    LastNamePrefix,
    #[serde(rename = "naam.geslachtsnaam")]
    LastName,
    #[serde(rename = "geboorte.datum")]
    DateOfBirth,
    #[serde(rename = "geslacht")]
    Gender,
    #[serde(rename = "naam.aanduidingNaamgebruik")]
    DesignatedNameUsage,

    // Eligibility to stand for election: article 56 of the Grondwet requires a
    // candidate to be a Dutch national, to have reached the age of eighteen and
    // not to be excluded from the right to vote.
    #[serde(rename = "overlijden.datum")]
    DateOfDeath,
    #[serde(rename = "nationaliteiten.nationaliteit")]
    Nationality,
    #[serde(rename = "uitsluitingKiesrecht")]
    SuffrageExclusion,

    // Place of residence. Only the `woonplaats` is verified: it is the one
    // address element printed on the candidate list. The rest of the address is
    // not, and the BRP models it differently than this application does.
    #[serde(rename = "verblijfplaats.verblijfadres.woonplaats")]
    PlaceOfResidence,

    // Municipality of registration
    // TODO: What to do with Registratie Niet Ingezetenen?
    #[serde(rename = "gemeenteVanInschrijving")]
    RegisteredMunicipality,
    #[serde(rename = "datumInschrijvingInGemeente")]
    DateMunicipalRegistration,

    // Partners
    #[serde(rename = "partners.naam.voorvoegsel")]
    PartnerLastNamePrefix,
    #[serde(rename = "partners.naam.geslachtsnaam")]
    PartnerLastName,
    #[serde(rename = "partners.aangaanHuwelijkPartnerschap.datum")]
    DateOfMarriage,
    #[serde(rename = "partners.ontbindingHuwelijkPartnerschap")]
    DateOfDissolutionMarriage,
}
