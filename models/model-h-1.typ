#import "layout.typ": checkbox, conf, enumerated_table, fill_in

#let input = json("./input.json")

#show: doc => conf(
  doc,
  "Model H 1",
  "Kandidatenlijst",
  "Met dit formulier stelt u, als inleveraar van de kandidatenlijst, kandidaten verkiesbaar voor een verkiezing.",
  input,
)


== 1. Verkiezing
Het gaat om de verkiezing van: *#input.election_name*


== 2. Kieskringen
De kandidatenlijst wordt ingeleverd voor:
#if input.electoral_districts.tag == "All" {
  [*alle kieskringen*]
} else {
  block(above: 1em, list(tight: true, ..input.electoral_districts.districts))
}


== 3. Aanduiding van de politieke groepering
Aanduiding boven de kandidatenlijst: *#input.designation*


== 4. Kandidaten op de lijst
#enumerated_table(
  columns: (1fr, 1fr, 1fr, 1fr),
  headers: ("naam", "voorletters", "geboortedatum", "woonplaats"),
  values: input.candidates.map(c => (c.last_name, c.initials, text(number-width: "tabular", datetime(..c.date_of_birth).display("[day]-[month]-[year]")), c.locality)),
)


== 5. Vervanger(s) voor het herstel van verzuimen
#if input.substitute_submitter.len() == 0 {
  [_geen_]
} else {
  enumerated_table(
    columns: (1fr, 1fr, 1fr, 0.75fr, 1.5fr),
    headers: ("naam", "voorletters", "postadres", "postcode", "plaats"),
    values: input.substitute_submitter.map(s => (
      s.last_name,
      s.initials,
      s.postal_address,
      s.postal_code,
      s.locality,
  )),
)
}

== 6. In te leveren bij de kandidatenlijst
Ik ben verplicht de volgende bijlage(n) in te leveren bij de kandidatenlijst:

#checkbox(
  checked: true,
)[Een verklaring van de gemachtigde(n) van de politieke groepering(en) waarmee aan mij toestemming wordt gegeven om de aanduiding boven de kandidatenlijst te plaatsen, want ik heb een aanduiding boven de lijst geplaatst (model H 3-1 of H 3-2).]
#checkbox(
  checked: not input.previously_seated,
)[Verklaringen van kiezers dat zij de lijst ondersteunen, want de lijst komt niet in aanmerking voor de ontheffing van deze verplichting (model H 4 dan wel model Pa 11; model Pa 11 wordt alleen gebruikt bij de verkiezing van het kiescollege voor niet-ingezetenen).]
#checkbox(
  checked: true,
)[Een verklaring van iedere op de lijst voorkomende kandidaat dat hij instemt met zijn kandidaatstelling op de lijst (model H 9).]
#checkbox(
  checked: true,
)[Een kopie van een geldig identiteitsbewijs van iedere kandidaat die géén zitting heeft in het orgaan waarvoor de verkiezing wordt gehouden.]
#checkbox(
  checked: not input.previously_seated,
)[Een betalingsbewijs van de waarborgsom, want de lijst komt niet in aanmerking voor de ontheffing van deze verplichting (model H 12).]
#if input.election_type != "EK" [
  #checkbox(
    checked: true,
  )[Een verklaring van voorgenomen vestiging van iedere op de lijst voorkomende kandidaat die niet woonachtig is in het gebied waarop de verkiezing betrekking heeft (alleen bij een verkiezing van provinciale staten, het algemeen bestuur van een waterschap, een gemeenteraad, de eilandsraden van de openbare lichamen Bonaire, Saba of Sint Eustatius en de kiescolleges van de openbare lichamen).]
  #checkbox(
    checked: true,
  )[Een verklaring van voorgenomen vestiging buiten Nederland van iedere op de lijst voorkomende kandidaat die woonachtig is in Nederland (alleen bij een verkiezing van het kiescollege voor niet-ingezetenen). #highlight[\@Iris, deze stond niet in de screenshot maar wel in de ODT]]
  #checkbox(
    checked: true,
  )[Een verklaring van iedere op de lijst voorkomende kandidaat dat hij niet in een andere lidstaat kandidaat zal zijn voor het Europees Parlement (model Y 13).]
  #checkbox(
    checked: true,
  )[Een verklaring van kandidaten die onderdaan zijn van een andere lidstaat, dat zij in die lidstaat niet zijn uitgesloten van het recht om gekozen te worden voor de verkiezingen van het Europees Parlement (model Y 35).]
]


== 7. Ondertekening door de inleveraar
#let submitter = input.list_submitter

#table(
  stroke: none,
  inset: 0.75em,
  align: horizon,
  columns: (auto, 1fr),
  "Naam en voorletters", [#submitter.last_name, #submitter.initials],
  "Postadres, postcode en plaats", [#submitter.postal_address, #submitter.postal_code #submitter.locality],
  "Datum", fill_in,
  "Handtekening", fill_in,
)
