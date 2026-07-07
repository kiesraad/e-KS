#import "layout.typ": checkbox, column_table, conf, date, fill_in, label_table, mono, translator

#let input = json("./input.json")
#let trans = translator(input.locale)
#show: doc => conf(
  doc,
  "Model H 3-2",
  trans(
    "Machtiging om samengevoegde aanduiding boven kandidatenlijst te plaatsen",
    "Machtiging om gearfoege oantsjutting boppe kandidatelist te pleatsen",
  ),
  trans[
    Met dit formulier geeft u de inleveraar van de kandidatenlijst toestemming om een aanduiding boven de kandidatenlijst te plaatsen, die is gevormd door samenvoeging van de aanduidingen van politieke groeperingen of afkortingen daarvan.

    U kunt alleen toestemming geven als u hiertoe gemachtigd bent door uw politieke groepering.
  ][
    Mei dit formulier jouwe jo dejinge dy’t de kandidatelist ynleveret tastimming om in oantsjutting boppe de kandidatelist te pleatsen, dy’t foarme is troch gearfoeging fan de oantsjuttings fan politike groepearrings of ôfkoartings dêrfan.

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

= #trans("Aanduiding van de politieke groeperingen", "Oantsjutting fan de politike groepearrings") <aanduiding>
#trans(
  "De samengevoegde aanduiding van de politieke groeperingen:",
  "De gearfoege oantsjutting fan de politike groepearrings:",
)
*#input.designation*


= #trans("Toestemming aan de inleveraar", "Tastimming oan dejinge dy’t ynleveret")
#let submitter = input.list_submitter
#trans(
  "Wij geven toestemming aan",
  "Wy jouwe tastimming oan",
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
  "Ondertekening door de gemachtigden",
  "Undertekening troch de lêsthawwer",
)
#for (i, name_authorisation) in input.name_authorisations.enumerate() [
  == #trans("Gemachtigde van politieke groepering", "Lêsthawwer fan politike groepearring") #(i + 1)
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
]
