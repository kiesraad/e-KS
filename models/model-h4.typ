#import "layout.typ": checkbox, column_table, conf, date, enumerated_table, fill_in, label_table, mono, translator


#let input = json("./input.json")

#let trans = translator(input.locale)

#show: doc => conf(
  doc,
  "Model H 4",
  trans(
    "Ondersteuningsverklaring",
    "Stipeferklearring",
  ),
  trans(
    [
      Met dit formulier verklaart u dat u een kandidatenlijst ondersteunt van een politieke groepering. Dit betekent dat u de deelname van de betreffende groepering aan de verkiezing mogelijk maakt. Deze verklaring wordt ter inzage gelegd.

      *Let op!*\
      U mag zich niet laten omkopen tot het afleggen van deze ondersteuningsverklaring. Degene die u omkoopt of u hiertoe anderszins dwingt, is tevens strafbaar. Op beide misdrijven staat een gevangenisstraf van maximaal zes maanden of een geldboete.
    ],
    [
      Mei dit formulier ferklearje jo dat jo in kandidatelist fan in politike groepearring stypje. Dat betsjut dat jo de dielname fan de oanbelangjende groepearring oan de ferkiezing mooglik meitsje. Dizze ferklearring wurdt op ynsjen lein.

      *Tink der om!*\
      Jo meie jo net omkeapje litte ta it ôflizzen fan dizze stipeferklearring. Dejinge dy't jo omkeapet of jo dêrta op oare wize twingt, is tagelyk strafber. Op beide misdriuwen stiet in finzenisstraf fan maksimaal seis moannen of in jildboete.
    ],
  ),
  page-label: (n, m) => trans([Pagina #n van #m], [Side #n fan #m]),
  input,
)

= #trans("Verkiezing", "Ferkiezing")
#trans(
  "Het gaat om de verkiezing van:",
  "It giet om de ferkiezing fan:",
)
*#input.election_name*

= #trans("Aanduiding van de politieke groepering", "Oantsjutting fan de politike groepearring")
#trans(
  "De aanduiding van de politieke groepering waarvan u de kandidatenlijst ondersteunt: ",
  "De oantsjutting fan de politike groepearring dêr't jo de kandidatelist fan stypje: ",
)
*#input.designation*

= #trans("Kandidaten op de lijst", "Kandidaten op de list")
#column_table(
  columns: (auto, 1fr, 1fr, 1fr),
  headers: (
    "",
    trans("naam", "namme"),
    trans("voorletters", "foarletters"),
    trans("woonplaats", "wenplak"),
  ),
  values: input.candidates.map(c => ([#c.position], c.last_name, c.initials, c.locality)),
)

= #trans("Ondertekening door de kiezer", "Undertekening troch de kiezer")
#trans(
  "Ik verklaar dat ik de bovengenoemde kandidatenlijst ondersteun.",
  "Ik ferklearje dat ik de boppeneamde kandidatelist stypje.",
)
#label_table(values: (
  (trans("Datum", "Datum"), fill_in()),
  (trans("Naam", "Namme"), fill_in()),
  (trans("Handtekening", "Hantekening"), fill_in(height: 4em)),
))

#let gr_or_other = (gr, non_gr) => if input.election_type == "GR" { gr } else { non_gr }
#if input.election_type != "EK" {
  [
    = #trans[
      Verklaring van de #gr_or_other("burgermeester", "gezaghebber")
    ][
      Ferklearring fan de #gr_or_other("boargemaster", "gesachhawwer")
    ]
    #trans[
      De #gr_or_other("burgemeester", "gezaghebber") van #fill_in(width: 15em) verklaart dat de ondersteuner in zijn #gr_or_other("gemeente", "openbaar lichaam") als kiezer is geregistreerd.
    ][
      De #gr_or_other("burgemeester", "gesachhawwer") fan #fill_in(width: 15em) ferklearret dat de stiper yn syn #gr_or_other("gemeente", "iepenbier lichem") as kiezer registrearre is.
    ]

    #label_table(values: (
      (
        trans(
          "De kiezer behoort tot kieskring",
          "De kiezer heart ta kiesrûnte",
        ),
        fill_in(),
      ),
      (trans("Datum", "Datum"), fill_in()),
      (trans("Ondertekening of gemeentestempel", "Undertekening of gemeentestimpel"), fill_in(height: 4em)),
    ))
  ]
}
