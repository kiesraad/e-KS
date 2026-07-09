#import "layout.typ": checkbox, column_table, conf, fill_in, plain_table

#let input = json("./input.json")

#show: doc => conf(
  doc,
  "Model I 4",
  "Proces-verbaal over geldigheid en nummering kandidatenlijsten",
  [
    Met dit formulier doet het centraal stembureau verslag van de zitting waarin is besloten over:
    - de geldigheid en nummering van de kandidatenlijsten;
    - het handhaven van de kandidaten op, en de aanduidingen bovenaan, de kandidatenlijsten.
  ],
  input,
)

#let placeholder = pad(left: 1em)[n.t.b.]


= Verkiezing
Het gaat om de verkiezing van *#input.election_name*

Dag van stemming *#input.election_date*


= Zitting
Het betreft de openbare zitting van het centraal stembureau in *#input.public_session.location*

Datum zitting *#input.public_session.date*

Tijdstip zitting *#input.public_session.time*


= Geconstateerde verzuimen
Bij het onderzoek naar de kandidatenlijsten waren
#if input.found_omissions.len() == 0 [
  geen herstelbare verzuimen geconstateerd.
] else [
  de volgende herstelbare verzuimen geconstateerd:
  #plain_table(
    columns: (1fr, 2fr),
    headers: ("Aanduiding in de kieskring(en)", "omschrijving verzuim"),
    values: input.found_omissions.map(p => p
      .omission_descriptions
      .enumerate()
      .map(((i, od)) => (
        if i != 0 { "" } else { p.designation + " in " + p.electoral_district },
        od,
      ))),
  )
]


= Herstelde verzuimen
#if input.recovered_omissions == none {
  placeholder
} else if input.recovered_omissions.len() == 0 [
  Er zijn geen verzuimen hersteld.
] else [
  De volgende verzuimen zijn hersteld:
  #plain_table(
    columns: (1fr, 2fr),
    headers: ("Aanduiding in de kieskring(en)", "omschrijving verzuim"),
    values: input.recovered_omissions.map(p => p
      .omission_descriptions
      .enumerate()
      .map(((i, od)) => (
        if i != 0 { "" } else { p.designation + " in " + p.electoral_district },
        od,
      ))),
  )
]


= Ongeldige lijsten
#if input.invalid_lists == none {
  placeholder
} else [
  Het centraal stembureau besluit dat
  #if input.invalid_lists.len() == 0 [
    geen lijst ongeldig is verklaard.
  ] else [
    de volgende lijsten ongeldig zijn verklaard:
    #plain_table(
      columns: (1fr, 2fr),
      headers: ("Aanduiding in de kieskring(en)", "omschrijving verzuim"),
      values: input.invalid_lists.map(p => p
        .omission_descriptions
        .enumerate()
        .map(((i, od)) => (
          if i != 0 { "" } else { p.designation + " in " + p.electoral_district },
          od,
        ))),
    )
  ]
]


= Geschrapte kandidaten
#if input.removed_candidates == none {
  placeholder
} else [
  Het centraal stembureau besluit dat
  #if input.removed_candidates.len() == 0 [
    geen kandidaat van een lijst is geschrapt.
  ] else [
    de volgende kandidaten van een lijst zijn geschrapt:
    #plain_table(
      columns: (1fr, 1fr, 2fr),
      headers: ("Aanduiding in de kieskring(en)", "naam kandidaat", "reden"),
      values: input.removed_candidates.map(p => p
        .candidates
        .enumerate()
        .map(((i, c)) => (
          if i != 0 { "" } else { p.designation + " in " + p.electoral_district },
          c.name,
          c.reason,
        ))),
    )
  ]
]


= Geschrapte aanduidingen
#if input.removed_designations == none {
  placeholder
} else [
  Het centraal stembureau besluit dat
  #if input.removed_designations.len() == 0 [
    geen aanduiding boven een lijst is geschrapt.
  ] else [
    de volgende aanduidingen boven een lijst zijn geschrapt:
    #plain_table(
      columns: (1fr, 1fr, 2fr),
      headers: ("Aanduiding in de kieskring(en)", "naam eerste kandidaat op de lijst", "reden"),
      values: input.removed_designations.map(p => (
        p.designation + " in " + p.electoral_district,
        p.first_candidate_name,
        p.reason,
      )),
    )
  ]
]


= Gecorrigeerde aanduiding
#if input.corrected_designations == none {
  placeholder
} else [
  Het centraal stembureau besluit dat
  #if input.corrected_designations.len() == 0 [
    geen aanduiding boven een lijst ambtshalve is aangepast.
  ] else [
    de volgende aanduidingen boven een lijst ambtshalve zijn aangepast:
    #plain_table(
      columns: (1fr, 1fr, 2fr),
      headers: (
        "Naam eerste kandidaat in de kieskring(en)",
        "vermelde aanduiding bij inlevering",
        "aangepaste aanduiding",
      ),
      values: input.corrected_designations.map(p => (
        p.first_candidate_name + " in " + p.electoral_district,
        p.submitted_designation,
        p.edited_designation,
      )),
    )
  ]
]


= Geldige lijsten
Het centraal stembureau besluit dat de volgende lijsten geldig zijn verklaard:

#if input.valid_lists == none {
  placeholder
} else {
  pagebreak(weak: true)
  for electoral_district in input.valid_lists [
    == Kieskring *#electoral_district.electoral_district*

    #for (i, list) in electoral_district.lists.enumerate() [
      === #numbering("A", i + 1). #list.designation
      #column_table(
        columns: (auto, 1fr, 1fr, 1fr),
        headers: (
          "",
          "naam kandidaat",
          "voorletters",
          "woonplaats",
        ),
        values: list.candidates.map(c => (
          [#c.position],
          c.last_name,
          c.initials,
          c.locality,
        )),
      )
      #pagebreak(weak: true)
    ]
  ]
}


= Nummering van de kandidatenlijsten
== Nummering op grond van het aantal stemmen behaald bij de laatstgehouden verkiezing
Eerst zijn de kandidatenlijsten genummerd van de politieke groeperingen die een of meer zetels hebben behaald bij de laatstgehouden verkiezing, in de volgorde van de bij die verkiezing op de desbetreffende lijsten uitgebrachte aantallen stemmen. Voor zover nodig is rekening gehouden met samengevoegde aanduidingen. Bij een gelijk aantal stemmen is er genummerd via loting.

#if input.numbered_based_on_votes == none {
  placeholder
} else {
  column_table(
    columns: (auto, 1fr, auto),
    align: (horizon, horizon, horizon + right),
    headers: (
      "nummer",
      "aanduiding politieke groepering",
      "aantal stemmen bij laatste verkiezing",
    ),
    values: input.numbered_based_on_votes.map(p => (
      [#p.position],
      p.designation,
      [#p.previous_votes],
    )),
  )
}

== Nummering van de overige lijsten
Vervolgens zijn de overige kandidatenlijsten genummerd in de volgorde van het aantal kieskringen waarvoor de lijst is ingeleverd. Bij een gelijk aantal kieskringen is er genummerd via loting.

#if input.numbered_based_on_districts == none {
  placeholder
} else {
  column_table(
    columns: (auto, 1fr, auto),
    align: (horizon, horizon, horizon + right),
    headers: (
      "nummer",
      "aanduiding politieke groepering of naam eerste kandidaat",
      "aantal kieskringen waarvoor lijst geldt",
    ),
    values: input.numbered_based_on_districts.map(p => (
      [#p.position],
      p.designation,
      [#p.districts],
    )),
  )
}


= Bezwaren van de aanwezige kiezers
Tijdens de zitting zijn
#if input.objections == none [
  #checkbox(checked: false)[geen bezwaren ingebracht.]
  #checkbox(checked: false)[de volgende bezwaren ingebracht:]
  #box(width: 100%, height: 30em, inset: 0pt)[] // room for writing
] else if input.objections.len() == 0 [
  geen bezwaren ingebracht.
] else [
  de volgende bezwaren ingebracht:
  #enum(numbering: "a.", ..input.objections.map(o => [#o]))
]
#if input.response_objections != none [
  #input.response_objections
]


= Ondertekening
#block(breakable: false, table(
  columns: (1.25fr, 1fr, 1.5fr),
  "Datum", input.public_session.date, none,
  "Naam en handtekening voorzitter", input.public_session.chair, fill_in(height: 4em),
  ..input
    .public_session
    .members
    .enumerate()
    .map(((i, member)) => (
      if i == 0 { "Naam en handtekening leden" } else { none },
      member,
      fill_in(height: 4em),
    ))
    .flatten(),
  inset: (left: 0pt),
))
