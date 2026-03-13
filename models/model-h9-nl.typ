#import "layout.typ": checkbox, column_table, conf, fill_in, label_table, mono

#let input = json("./input.json")

#show: doc => conf(
  doc,
  "Model H 9",
  "Instemmingsverklaring",
  [
    Met dit formulier stemt u ermee in dat u op onderstaande kandidatenlijst staat, en u stemt in met uw positie op die lijst.

    *Let op!* Bent u nog geen lid van het vertegenwoordigend orgaan? Voeg dan een kopie van een geldig identiteitsbewijs bij.
  ],
  page-label: (n, m) => [Pagina #n van #m],
  input,
)


= Verkiezing
Het gaat om de verkiezing van: *#input.election_name*


= Kieskringen
// TODO: this is slightly different from the h1, should we allow people to only agree for certain electoral districts?
Mijn instemming geldt voor:
#if input.electoral_districts.tag == "All" {
  [*alle kieskringen*]
} else {
  block(above: 1em, list(tight: true, ..input.electoral_districts.districts))
}


= Politieke groepering
De aanduiding van de politieke groepering waarvan de kandidatenlijst is: *#input.designation*


= Kandidaten op de lijst
#column_table(
  columns: (auto, 1fr, 1fr, 1fr),
  headers: ("", "naam", "voorletters", "woonplaats"),
  values: input.candidates.map(c => ([#c.position], c.last_name, c.initials, c.locality)),
)


= Gemachtigde voor het aannemen van uw benoeming
#if input.detailed_candidate.representative == none {
  [_niet van toepassing_]
} else {
  column_table(
    columns: (1fr, 1fr, 1fr, 0.75fr, 1.5fr),
    headers: ("naam", "voorletters", "postadres", "postcode", "plaats"),
    values: (
      (
        input.detailed_candidate.representative.last_name,
        input.detailed_candidate.representative.initials,
        input.detailed_candidate.representative.postal_address.street_address,
        mono(input.detailed_candidate.representative.postal_address.postal_code),
        input.detailed_candidate.representative.postal_address.locality,
      ),
    ),
  )
}


#if input.election_type != "KNCI" [
  = Adres voor de kennisgeving van mijn benoeming
  // deze rubriek is niet van toepassing bij de verkiezing van het kiescollege voor niet-ingezetenen
  #if input.detailed_candidate.postal_address == none {
    [_niet van toepassing_]
  } else {
    column_table(
      columns: (1fr, 0.5fr, 1fr),
      headers: ("postadres", "postcode", "plaats"),
      values: (
        (
          input.detailed_candidate.postal_address.street_address,
          mono(input.detailed_candidate.postal_address.postal_code),
          input.detailed_candidate.postal_address.locality,
        ),
      ),
    )
  }
]

#if input.election_type == "KCNI" and input.detailed_candidate.authorized_agent == none [
  = Kennisgeving van mijn benoeming ontvangen langs digitale weg
  #checkbox(checked: false)[
    Ik stem ermee in de kennisgeving van mijn benoeming te ontvangen via een digitale berichtenbox waartoe ik toegang kan krijgen met gebruikmaking van een DigiD. Hierbij bevestig ik tevens dat ik een DigiD zal aanvragen indien ik hier nog niet over beschik.
  ]
]

= Ondertekening door de kandidaat
#label_table(values: (
  ("Naam", [#input.detailed_candidate.candidate.last_name, #input.detailed_candidate.candidate.initials]),
  ("Woonplaats", input.detailed_candidate.candidate.locality),
  ("Burgerservicenummer", input.detailed_candidate.bsn),
  ("Datum", fill_in()),
  ("Handtekening", fill_in(height: 4em)),
))
