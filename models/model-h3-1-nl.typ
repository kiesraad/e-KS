#import "layout.typ": checkbox, column_table, conf, date, enumerated_table, fill_in, label_table, mono

#let input = json("./input.json")
#show: doc => conf(
  doc,
  "Model H 3-1",
  "Machtiging om aanduiding boven kandidatenlijst te plaatsen",
  [
    Met dit formulier geeft u de inleveraar van de kandidatenlijst toestemming om de aanduiding die door uw politieke groepering is geregistreerd boven de kandidatenlijst te plaatsen.

    U kunt alleen toestemming geven als u hiertoe gemachtigd bent door uw politieke groepering.
  ],
  page-label: (n, m) => [Pagina #n van #m],
  input,
)

= Verkiezing
Het gaat om de kandidatenlijst voor de verkiezingen van: *#input.election_name*

= Kieskringen
De machtiging geldt
#if input.electoral_districts.tag == "All" {
  [*voor alle kieskringen waarvoor de kandidatenlijst wordt ingeleverd.*]
} else {
  [*uitsluitend voor de volgende kieskring(en):*]
  block(above: 1em, list(tight: true, ..input.electoral_districts.districts))
}

= Aanduiding van de politieke groepering
De geregistreerde aanduiding van de politieke groepering: *#input.designation*

= Toestemming aan de inleveraar
#let submitter = input.list_submitter
Ik geef toestemming aan *#submitter.last_name, #submitter.initials (#submitter.first_name)* om de onder punt 3 vermelde aanduiding boven de kandidatenlijst te plaatsen.

= Kandidaten op de lijst
#column_table(
  columns: (auto, 1fr, 1fr, 1fr),
  headers: ("", "naam", "voorletters", "woonplaats"),
  values: input.candidates.map(c => ([#c.position], c.last_name, c.initials, c.locality)),
)
= Ondertekening door de gemachtigde van de politieke groepering
#let agent = input.authorised_agent
#label_table(values: (
  ("Datum", fill_in()),
  ("Naam van de gemachtigde van de politieke groepering", [#agent.last_name, #agent.initials (#agent.first_name)]),
  ("Volledige statutaire naam van de politieke groepering", [#input.legal_name]),
  ("Handtekening", fill_in(height: 4em)),
))
