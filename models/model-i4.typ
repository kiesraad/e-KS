#import "layout.typ": conf, plain_table, column_table

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

= Verkiezing
Het gaat om de verkiezing van *#input.election.name*

Dag van stemming *#input.election.date*

= Zitting
Het betreft de openbare zitting van het centraal stembureau in *#input.session.location*

Datum zitting *#input.session.date*

Tijdstip zitting *#input.session.time*

= Geconstateerde verzuimen

Bij het onderzoek naar de kandidatenlijsten waren
#if input.found_omissions.len() == 0 {
  "geen herstelbare verzuimen geconstateerd."
} else {
  [
    de volgende herstelbare verzuimen geconstateerd:
    #plain_table(
      columns: (1fr, 2fr),
      headers: ("Aanduiding in de kieskring(en)", "omschrijving verzuim"),
      values: input.found_omissions.map(p => p
        .omission_descriptions
        .enumerate()
        .map(((i, od)) => (
          if i != 0 { "" } else { p.appellation + " in " + p.electoral_district },
          od,
        ))),
    )
  ]
}

= Herstelde verzuimen
#if input.recovered_omissions.len() == 0 {
  "Er zijn geen verzuimen hersteld"
} else {
  [
    De volgende verzuimen zijn hersteld:
    #plain_table(
      columns: (1fr, 2fr),
      headers: ("Aanduiding in de kieskring(en)", "omschrijving verzuim"),
      values: input.recovered_omissions.map(p => p
        .omission_descriptions
        .enumerate()
        .map(((i, od)) => (
          if i != 0 { "" } else { p.appellation + " in " + p.electoral_district },
          od,
        ))),
    )
  ]
}

= Ongeldige lijsten
Het centraal stembureau besluit dat
#if input.invalid_lists.len() == 0 {
  "geen lijst ongeldig is verklaard."
} else {
  [
    de volgende lijsten ongeldig zijn verklaard:
    #plain_table(
      columns: (1fr, 2fr),
      headers: ("Aanduiding in de kieskring(en)", "omschrijving verzuim"),
      values: input.invalid_lists.map(p => p
        .omission_descriptions
        .enumerate()
        .map(((i, od)) => (
          if i != 0 { "" } else { p.appellation + " in " + p.electoral_district },
          od,
        ))),
    )
  ]
}

= Geschrapte kandidaten
Het centraal stembureau besluit dat
#if input.removed_candidates.len() == 0 {
  "geen kandidaat van een lijst is geschrapt."
} else {
  [
    de volgende kandidaten van een lijst zijn geschrapt:
    #plain_table(
      columns: (1fr, 1fr, 2fr),
      headers: ("Aanduiding in de kieskring(en)", "naam kandidaat", "reden"),
      values: input.removed_candidates.map(p => p
        .candidates
        .enumerate()
        .map(((i, c)) => (
          if i != 0 { "" } else { p.appellation + " in " + p.electoral_district },
          c.name,
          c.reason,
        ))),
    )
  ]
}

= Geschrapte aanduidingen
Het centraal stembureau besluit dat
#if input.removed_appellations.len() == 0 {
  "geen aanduiding boven een lijst is geschrapt."
} else {
  [
    de volgende aanduidingen boven een lijst zijn geschrapt:
    #plain_table(
      columns: (1fr, 1fr, 2fr),
      headers: ("Aanduiding in de kieskring(en)", "naam eerste kandidaat op de lijst", "reden"),
      values: input.removed_appellations.map(p => (
        p.appellation + " in " + p.electoral_district,
        p.first_candidate_name,
        p.reason,
      )),
    )
  ]
}

= Gecorrigeerde aanduiding
Het centraal stembureau besluit dat
#if input.corrected_appellations.len() == 0 {
  "geen aanduiding boven een lijst ambtshalve is aangepast."
} else {
  [
    de volgende aanduidingen boven een lijst ambtshalve zijn aangepast:
    #plain_table(
      columns: (1fr, 1fr, 2fr),
      headers: (
        "Naam eerste kandidaat in de kieskring(en)",
        "vermelde aanduiding bij inlevering",
        "aangepaste aanduiding",
      ),
      values: input.corrected_appellations.map(p => (
        p.first_candidate_name + " in " + p.electoral_district,
        p.submitted_appellation,
        p.edited_appellation,
      )),
    )
  ]
}

= Geldige lijsten
Het centraal stembureau besluit dat de volgende lijsten geldig zijn verklaard:
#pagebreak()
#let alphabet = "ABCDEFGHIJKLMNOPQRSTUVWXYZ".clusters()

#for electoral_district in input.valid_lists {
  [
    Kieskring *#electoral_district.electoral_district*

    #for (i, list) in electoral_district.lists.enumerate() {
      let list_letter = alphabet.at(i)
      [
        #list_letter *#list.appellation*
        #column_table(
          columns: (auto, 1fr, 1fr, 1fr),
          headers: (
            "",
            "naam kandidaat",
            "voorletters",
            "woonplaats"
          ),
          values: list.candidates.map(c => (
            [#c.position],
            c.last_name,
            c.initials,
            c.locality,
          )),
        )
        #pagebreak()
      ]
    }
  ]
}
= Nummering van de kandidatenlijsten
== Nummering op grond van het aantal stemmen behaald bij de laatstgehouden verkiezing
Eerst zijn de kandidatenlijsten genummerd van de politieke groeperingen die een of meer zetels hebben behaald bij de laatstgehouden verkiezing, in de volgorde van de bij die verkiezing op de desbetreffende lijsten uitgebrachte aantallen stemmen. Voor zover nodig is rekening gehouden met samengevoegde aanduidingen. Bij een gelijk aantal stemmen is er genummerd via loting.

== Nummering van de overige lijsten
Vervolgens zijn de overige kandidatenlijsten genummerd in de volgorde van het aantal kieskringen waarvoor de lijst is
ingeleverd. Bij een gelijk aantal kieskringen is er genummerd via loting.
= Bezwaren van de aanwezige kiezers

= Ondertekening
