#import "layout.typ": checkbox, column_table, conf, date, fill_in, label_table, mono, translator

#let input = json("./input.json")
#let trans = translator(input.locale)
#show: doc => conf(
  doc,
  "Model H 3-1",
  trans(
    "Machtiging om aanduiding boven kandidatenlijst te plaatsen",
    "Machtiging om oantsjutting boppe kandidatelist te pleatsen",
  ),
  trans[
    Met dit formulier geeft u de inleveraar van de kandidatenlijst toestemming om de aanduiding die door uw politieke groepering is geregistreerd boven de kandidatenlijst te plaatsen.

    U kunt alleen toestemming geven als u hiertoe gemachtigd bent door uw politieke groepering.
  ][
    Mei dit formulier jouwe jo dejinge dy’t de kandidatelist ynleveret tastimming om de oantsjutting dy’t troch jo politike groepearring registrearre is boppe de kandidatelist te pleatsen.

    Jo kinne allinnich tastimming jaan as jo dêrta machtige binne troch jo politike groepearring.
  ],
  input,
)


= #trans("Verkiezing", "Ferkiezing")
#trans(
  "Het gaat om de kandidatenlijst voor de verkiezingen van:",
  "It giet om de kandidatelist foar de ferkiezing fan:",
)
*#input.election_name*

#if input.electoral_districts.tag != "OnlyOne" [
  = #trans("Kieskringen", "Kiesrûnten")
  #trans("De machtiging geldt", "De machtiging jildt")
  #if input.electoral_districts.tag == "All" {
    trans(
      [*voor alle kieskringen waarvoor de kandidatenlijst wordt ingeleverd.*],
      [*foar alle kiesrûnten dêr’t de kandidatelist foar ynlevere wurdt.*],
    )
  } else {
    trans(
      [*uitsluitend voor de volgende kieskring(en):*],
      [*allinnich foar de neikommende kiesrûnte(n):*],
    )
    block(above: 1em, input.electoral_districts.districts.join(", "))
  }
]

= #trans("Aanduiding van de politieke groepering", "Oantsjutting fan de politike groepearring") <aanduiding>
#trans(
  "De geregistreerde aanduiding van de politieke groepering:",
  "De registrearre oantsjutting fan de politike groepearring:",
)
*#input.designation*


= #trans("Toestemming aan de inleveraar", "Tastimming oan dejinge dy’t ynleveret")
#let submitter = input.list_submitter
#trans(
  "Ik geef toestemming aan",
  "Ik jou tastimming oan",
)
*#submitter.last_name, #submitter.initials*
#trans[
  om de onder punt @aanduiding vermelde aanduiding boven de kandidatenlijst te plaatsen.
][
  om de ûnder punt @aanduiding neamde oantsjutting boppe de kandidatelist te pleatsen.
]


= #trans("Kandidaten op de lijst", "Kandidaten op de list")
#column_table(
  columns: (auto, 1fr, 1fr, 1fr),
  headers: ("", trans("naam", "namme"), trans("voorletters", "foarletters"), trans("woonplaats", "wenplak")),
  values: input.candidates.map(c => ([#c.position], c.last_name, c.initials, c.locality)),
)


= #trans(
  "Ondertekening door de gemachtigde van de politieke groepering",
  "Undertekening troch de lêsthawwer fan de politike groepearring",
)
#let name_authorisation = input.name_authorisations.at(0)
#label_table(values: (
  (trans("Datum", "Datum"), fill_in()),
  (
    trans(
      "Naam van de gemachtigde van de politieke groepering",
      "Namme fan de lêsthawwer fan de politike groepearring",
    ),
    (name_authorisation.last_name, name_authorisation.initials).filter(x => x != "").join(", "),
  ),
  (
    trans(
      "Volledige statutaire naam van de politieke groepering",
      "Folsleine statutêre namme fan de politike groepearring",
    ),
    [#name_authorisation.legal_name],
  ),
  (trans("Handtekening", "Hantekening"), fill_in(height: 4em)),
))
