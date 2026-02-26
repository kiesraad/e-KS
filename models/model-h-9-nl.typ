#import "layout.typ": checkbox, column_table, conf, enumerated_table, fill_in, label_table, mono

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


== 1. Verkiezing
Het gaat om de verkiezing van: *#input.election_name*


== 2. Kieskringen
De kandidatenlijst wordt ingeleverd voor:
#if input.electoral_districts == none {
  [*alle kieskringen*]
} else {
  block(above: 1em, list(tight: true, ..input.electoral_districts))
}


== 3. Politieke groepering
De aanduiding van de politieke groepering waarvan de kandidatenlijst is: *#input.designation*


== 4. Kandidaten op de lijst
#enumerated_table(
  columns: (1fr, 1fr, 1fr),
  headers: ("naam", "voorletters", "woonplaats"),
  values: input.candidates.map(c => (c.last_name, c.initials, c.locality)),
)


== 5. Gemachtigde voor het aannemen van uw benoeming
#if input.candidate.authorized_agent == none {
  [_niet van toepassing_]
} else {
  column_table(
    columns: (1fr, 1fr, 1fr, 0.75fr, 1.5fr),
    headers: ("naam", "voorletters", "postadres", "postcode", "plaats"),
    values: (
      input.candidate.authorized_agent.last_name,
      input.candidate.authorized_agent.initials,
      input.candidate.authorized_agent.postal_address,
      mono(input.candidate.authorized_agent.postal_code),
      input.candidate.authorized_agent.locality,
    ),
  )
}


== 6. Adres voor de kennisgeving van mijn benoeming
// deze rubriek is niet van toepassing bij de verkiezing van het kiescollege voor niet-ingezetenen
#if input.candidate.postal_address == none {
  [_niet van toepassing_]
} else {
  column_table(
    columns: (1fr, 0.5fr, 1fr),
    headers: ("postadres", "postcode", "plaats"),
    values: (
      input.candidate.postal_address.postal_address,
      mono(input.candidate.postal_address.postal_code),
      input.candidate.postal_address.locality,
    ),
  )
}

== 7. Kennisgeving van mijn benoeming ontvangen langs digitale weg
// deze rubriek is alleen van toepassing bij de verkiezing van het kiescollege voor niet-ingezetenen
#if input.candidate.authorized_agent == none {
  checkbox(checked: false)[
    Ik stem ermee in de kennisgeving van mijn benoeming te ontvangen via een digitale berichtenbox waartoe ik toegang kan krijgen met gebruikmaking van een DigiD. Hierbij bevestig ik tevens dat ik een DigiD zal aanvragen indien ik hier nog niet over beschik.
  ]
} else {
  [_niet van toepassing_]
}

== 8. Ondertekening door de kandidaat
#label_table(values: (
  ("Naam", [#input.candidate.last_name, #input.candidate.initials]),
  ("Woonplaats", input.candidate.locality),
  ("Burgerservicenummer", input.candidate.bsn),
  ("Datum", fill_in),
  ("Handtekening", fill_in),
))
