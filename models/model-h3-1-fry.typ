#import "layout.typ": checkbox, column_table, conf, date, enumerated_table, fill_in, label_table, mono

#let input = json("./input.json")
#show: doc => conf(
  doc,
  "Model H 3-1",
  "Machtiging om oantsjutting boppe kandidatelist te pleatsen",
  [
    Mei dit formulier jouwe jo dejinge dy’t de kandidatelist ynleveret tastimming om de oantsjutting dy’t troch jo politike groepearring registrearre is boppe de kandidatelist te pleatsen.

    Jo kinne allinnich tastimming jaan as jo dêrta machtige binne troch jo politike groepearring.
  ],
  page-label: (n, m) => [Side #n fan #m],
  input,
)

= Ferkiezing
It giet om de kandidatelist foar de ferkiezing fan: *#input.election_name*

= Kiesrûnten
De machtiging jildt
#if input.electoral_districts.tag == "All" {
  [*foar alle kiesrûnten dêr’t de kandidatelist foar ynlevere wurdt.*]
} else {
  [*allinnich foar de neikommende kiesrûnte(n):*]
  block(above: 1em, list(tight: true, ..input.electoral_districts.districts))
}

= Oantsjutting fan de politike groepearring
De registrearre oantsjutting fan de politike groepearring: *#input.designation*

= Tastimming oan dejinge dy’t ynleveret
#let submitter = input.list_submitter
Ik jou tastimming oan *#submitter.last_name, #submitter.initials (#submitter.first_name)* om de ûnder punt 3 neamde oantsjutting boppe de kandidatelist te pleatsen.

= Kandidaten op de list
#column_table(
  columns: (auto, 1fr, 1fr, 1fr),
  headers: ("", "namme", "foarletters", "wenplak"),
  values: input.candidates.map(c => ([#c.position], c.last_name, c.initials, c.locality)),
)
= Undertekening troch de lêsthawwer fan de politike groepearring
#let agent = input.authorised_agent
#label_table(values: (
  ("Datum", fill_in()),
  ("Namme fan de lêsthawwer fan de politike groepearring", [#agent.last_name, #agent.initials (#agent.first_name)]),
  ("Folsleine statutêre namme fan de politike groepearring", [#input.legal_name]),
  ("Hantekening", fill_in(height: 4em)),
))
