#import "layout.typ": checkbox, column_table, conf, fill_in, label_table, mono

#let input = json("./input.json")
#let trans = (dutch, frisian) => if input.locale == "fry" { frisian } else { dutch }
#show: doc => conf(
  doc,
  "Model H 9",
  trans("Instemmingsverklaring", "Ynstimmingsferklearring"),
  trans(
    [
      Met dit formulier stemt u ermee in dat u op onderstaande kandidatenlijst staat, en u stemt in met uw positie op die lijst.

      *Let op!* Bent u nog geen lid van het vertegenwoordigend orgaan? Voeg dan een kopie van een geldig identiteitsbewijs bij.
    ],
    [
      Mei dit formulier stimme jo dermei yn dat jo op ûndersteande kandidatelist steane en jo ynstimme mei jo posysje op dy list.

      *Tink der om!* Binne jo noch gjin lid fan it fertsjintwurdigjend orgaan? Foegje dan in kopy fan in jildich identiteitsbewiis ta.
    ],
  ),
  page-label: (n, m) => trans([Pagina #n van #m], [Side #n fan #m]),
  input,
)


= #trans("Verkiezing", "Ferkiezing")
#trans(
  "Het gaat om de verkiezing van",
  "It giet om de ferkiezing fan",
)
*#input.election_name*


= #trans("Kieskringen", "Kiesrûnten")
// TODO: this is slightly different from the h1, should we allow people to only agree for certain electoral districts?
#trans("Mijn instemming geldt voor:", "Myn ynstimming jildt foar:")
#if input.electoral_districts.tag == "All" {
  trans([*alle kieskringen*], [*alle kiesrûnten*])
} else {
  block(above: 1em, list(tight: true, ..input.electoral_districts.districts))
}


= #trans("Politieke groepering", "Politike groepearring")
#trans(
  "De aanduiding van de politieke groepering waarvan de kandidatenlijst is:",
  "De oantsjutting fan de politike groepearring dêr’t de kandidatelist fan is:",
)
*#input.designation*


= #trans("Kandidaten op de lijst", "Kandidaten op de list")
#column_table(
  columns: (auto, 1fr, 1fr, 1fr),
  headers: ("", trans("naam", "namme"), trans("voorletters", "foarletters"), trans("woonplaats", "wenplak")),
  values: input.candidates.map(c => ([#c.position], c.last_name, c.initials, c.locality)),
)


= #trans("Gemachtigde voor het aannemen van uw benoeming", "Lêsthawwer foar it oannimmen fan jo beneaming")
#if input.detailed_candidate.representative == none {
  trans([_niet van toepassing_], [_net fan tapassing_])
} else {
  column_table(
    columns: (1fr, 1fr, 1fr, 0.75fr, 1.5fr),
    headers: (
      trans("naam", "namme"),
      trans("voorletters", "foarletters"),
      trans("postadres", "postadres"),
      trans("postcode", "postkoade"),
      trans("plaats", "plak"),
    ),
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
  = #trans("Adres voor de kennisgeving van mijn benoeming", "Adres foar de meidieling fan myn beneaming")
  // deze rubriek is niet van toepassing bij de verkiezing van het kiescollege voor niet-ingezetenen
  #if input.detailed_candidate.postal_address == none {
    trans([_niet van toepassing_], [_net fan tapassing_])
  } else {
    column_table(
      columns: (1fr, 0.5fr, 1fr),
      headers: (trans("postadres", "postadres"), trans("postcode", "postkoade"), trans("plaats", "plak")),
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

#if input.election_type == "KCNI" and input.detailed_candidate.representative == none [
  = #trans(
    "Kennisgeving van mijn benoeming ontvangen langs digitale weg",
    "Kennisjouwing fan myn beneaming fia digitale wei tasjoerd krije",
  )
  #checkbox(checked: false)[
    #trans(
      "Ik stem ermee in de kennisgeving van mijn benoeming te ontvangen via een digitale berichtenbox waartoe ik toegang kan krijgen met gebruikmaking van een DigiD. Hierbij bevestig ik tevens dat ik een DigiD zal aanvragen indien ik hier nog niet over beschik.",
      "Ik stim dermei yn dat de kennisjouwing fan myn beneaming my tastjoerd wurdt fia in digitale berjochteboks dêr’t ik tagong ta krije kin mei in DigiD. Ek befêstigje ik dat ik in DigiD oanfreegje sil as ik dy noch net ha.",
    )
  ]
]

= #trans("Ondertekening door de kandidaat", "Undertekening troch de kandidaat")
#label_table(values: (
  (
    trans("Naam", "Namme"),
    [#input.detailed_candidate.candidate.last_name, #input.detailed_candidate.candidate.initials],
  ),
  (trans("Woonplaats", "Wenplak"), input.detailed_candidate.candidate.locality),
  (trans("Burgerservicenummer", "Boargerservicenûmer"), input.detailed_candidate.bsn),
  (trans("Datum", "Datum"), fill_in()),
  (trans("Handtekening", "Hantekening"), fill_in(height: 4em)),
))
