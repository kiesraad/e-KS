#import "layout.typ": checkbox, column_table, conf, fill_in, label_table, mono

#let input = json("./input.json")

#show: doc => conf(
  doc,
  "Model H 9",
  "Ynstimmingsferklearring",
  [
    Mei dit formulier stimme jo dermei yn dat jo op ûndersteande kandidatelist steane en jo ynstimme mei jo posysje op dy list.

    *Tink der om!* Binne jo noch gjin lid fan it fertsjintwurdigjend orgaan? Foegje dan in kopy fan in jildich identiteitsbewiis ta.
  ],
  page-label: (n, m) => [Side #n fan #m],
  input,
)


= Ferkiezing
It giet om de ferkiezing fan: *#input.election_name*


= Kiesrûnten
Myn ynstimming jildt foar:
#if input.electoral_districts.tag == "All" {
  [*alle kiesrûnten*]
} else {
  block(above: 1em, list(tight: true, ..input.electoral_districts.districts))
}


= Politike groepearring
De oantsjutting fan de politike groepearring dêr’t de kandidatelist fan is: *#input.designation*


= Kandidaten op de list
#column_table(
  columns: (auto, 1fr, 1fr, 1fr),
  headers: ("", "namme", "foarletters", "wenplak"),
  values: input.candidates.map(c => ([#c.position], c.last_name, c.initials, c.locality)),
)


= Lêsthawwer foar it oannimmen fan jo beneaming
#if input.detailed_candidate.representative == none {
  [_net fan tapassing_]
} else {
  column_table(
    columns: (1fr, 1fr, 1fr, 0.75fr, 1.5fr),
    headers: ("namme", "foarletters", "postadres", "postkoade", "plak"),
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
  = Adres foar de meidieling fan myn beneaming
  // deze rubriek is niet van toepassing bij de verkiezing van het kiescollege voor niet-ingezetenen
  #if input.detailed_candidate.postal_address == none {
    [_net fan tapassing_]
  } else {
    column_table(
      columns: (1fr, 0.5fr, 1fr),
      headers: ("postadres", "postkoade", "plak"),
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

#if input.election_type == "KCNI" and input.candidate.authorized_agent == none [
  = Kennisjouwing fan myn beneaming fia digitale wei tasjoerd krije
  #checkbox(checked: false)[
    Ik stim dermei yn dat de kennisjouwing fan myn beneaming my tastjoerd wurdt fia in digitale berjochteboks dêr’t ik tagong ta krije kin mei in DigiD. Ek befêstigje ik dat ik in DigiD oanfreegje sil as ik dy noch net ha.
  ]
]

= Undertekening troch de kandidaat
#label_table(values: (
  ("Namme", [#input.detailed_candidate.candidate.last_name, #input.detailed_candidate.candidate.initials]),
  ("Wenplak", input.detailed_candidate.candidate.locality),
  ("Boargerservicenûmer", input.detailed_candidate.bsn),
  ("Datum", fill_in()),
  ("Hantekening", fill_in(height: 4em)),
))
