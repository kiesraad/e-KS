use serde::Serialize;

// ontbreekt: Aanduiding bijzonder Nederlanderschap
// ontbreekt: Ingangsdatum geldigheid met betrekking tot de elementen van de categorie Nationaliteit
#[derive(Debug, Serialize)]
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

    // Nationaliteit
    #[serde(rename = "nationaliteiten.nationaliteit")]
    Nationality,

    // Partners
    #[serde(rename = "partners.naam.voorvoegsel")]
    PartnerLastNamePrefix,
    #[serde(rename = "partners.naam.geslachtsnaam")]
    PartnerLastName,
    #[serde(rename = "partners.aangaanHuwelijkPartnerschap.datum")]
    DateOfMarriage,
    #[serde(rename = "partners.ontbindingHuwelijkPartnerschap")]
    DateOfDissolutionMarriage,

    // Date of death
    #[serde(rename = "overlijden.datum")]
    DateOfDeath,

    // Place of residence
    // TODO: What to do with Registratie Niet Ingezetenen?
    #[serde(rename = "gemeenteVanInschrijving")]
    RegisteredMunicipality,
    #[serde(rename = "datumInschrijvingInGemeente")]
    DateMunicipalRegistration,
    #[serde(rename = "verblijfplaats.verblijfadres.korteStraatnaam")]
    StreetName,
    #[serde(rename = "verblijfplaats.verblijfadres.huisnummer")]
    HouseNumber,
    #[serde(rename = "verblijfplaats.verblijfadres.huisletter")]
    HouseLetter,
    #[serde(rename = "verblijfplaats.verblijfadres.huisnummertoevoeging")]
    HouseNumberAddition,
    #[serde(rename = "verblijfplaats.verblijfadres.postcode")]
    PostalCode,
    #[serde(rename = "verblijfplaats.verblijfadres.woonplaats")]
    PlaceOfResidence,

    // Not sure if these are correct. They should be specifically foreign, but they
    // may also apply to interior addresses
    #[serde(rename = "verblijfplaats.verblijfadres.land")]
    CountryOfResidence, // Land adres buitenland
    #[serde(rename = "verblijfplaats.datumVan")]
    ResidenceDateFrom, // Datum aanvang adres buitenland
    #[serde(rename = "verblijfplaats.verblijfadres.regel1")]
    AddressLine1, // Regel 1 adres buitenland
    #[serde(rename = "verblijfplaats.verblijfadres.regel2")]
    AddressLine2, // Regel 2 adres buitenland
    #[serde(rename = "verblijfplaats.verblijfadres.regel3")]
    AddressLine3, // Regel 3 adres buitenland
}
