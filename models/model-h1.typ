#import "layout.typ": checkbox, column_table, conf, date, enumerated_table, fill_in, label_table, mono, translator

#let input = json("./input.json")
#let trans = translator(input.locale)
#show: doc => conf(
  doc,
  "Model H 1",
  trans("Kandidatenlijst", "Kandidatelist"),
  trans(
    "Met dit formulier stelt u, als inleveraar van de kandidatenlijst, kandidaten verkiesbaar voor een verkiezing.",
    "Mei dit formulier stelle jo, as dejinge dy’t de kandidatelist ynleveret, kandidaten ferkiesber foar in ferkiezing.",
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
#trans("De kandidatenlijst wordt ingeleverd voor", "De kandidatelist wurdt ynlevere foar")
#if input.electoral_districts.tag == "All" {
  trans([*alle kieskringen.*], [*alle kiesrûnten.*])
} else {
  trans([*de volgende kieskring(en):*], [*de neikommende kiesrûnte(n):*])
  block(above: 1em, input.electoral_districts.districts.join(", "))
}


= #trans("Aanduiding van de politieke groepering", "De politike groepearring")
#trans(
  "Aanduiding boven de kandidatenlijst:",
  "De politike groepearring dêr’t jo de kandidatelist fan stypje:",
)
*#input.designation*


= #trans("Kandidaten op de lijst", "Kandidaten op de list")
#column_table(
  columns: (auto, 1fr, 1fr, 1fr, 1fr),
  headers: (
    "",
    trans("naam", "namme"),
    trans("voorletters", "foarletters"),
    trans("geboortedatum", "bertedatum"),
    trans("woonplaats", "wenplak"),
  ),
  values: input.candidates.map(c => (
    [#c.position],
    c.last_name,
    c.initials,
    date(c.date_of_birth),
    c.locality,
  )),
)


= #trans("Vervanger(s) voor het herstel van verzuimen", "Ferfanger(s) foar it ferhelpen fan fersommen")
#if input.substitute_submitter.len() == 0 {
  trans([_geen_], [_geen_])
} else {
  enumerated_table(
    columns: (1fr, 1fr, 1fr, 0.75fr, 1.5fr),
    headers: (
      trans("naam", "namme"),
      trans("voorletters", "foarletters"),
      trans("postadres", "postadres"),
      trans("postcode", "postkoade"),
      trans("plaats", "plak"),
    ),
    values: input.substitute_submitter.map(s => (
      s.last_name,
      s.initials,
      s.postal_address.street_address,
      mono(s.postal_address.postal_code),
      s.postal_address.locality,
    )),
  )
}


= #trans("In te leveren bij de kandidatenlijst", "Yn te leverjen by de kandidatelist")
#trans(
  "Ik ben verplicht de volgende bijlage(n) in te leveren bij de kandidatenlijst:",
  "Ik bin ferplichte de neikommende taheakke by de kandidatelist yn te leverjen:",
)

#checkbox(checked: true)[
  #trans(
    "Een verklaring van de gemachtigde(n) van de politieke groepering(en) waarmee aan mij toestemming wordt gegeven om de aanduiding boven de kandidatenlijst te plaatsen, want ik heb een aanduiding boven de lijst geplaatst (model H 3-1 of H 3-2).",
    "In ferklearring fan de lêsthawwer(s) fan de politike groepearring(s) dêr’t my tastimming mei jûn wurdt om de oantsjutting boppe de kandidatelist te pleatsen, want ik haw in oantsjutting boppe de list pleatst (model H 3-1 of H 3-2).",
  )
]
#checkbox(checked: not input.previously_seated)[
  #trans(
    [Verklaringen van kiezers dat zij de lijst ondersteunen, want de lijst komt niet in aanmerking voor de ontheffing van deze verplichting (#if input.election_type == "KCNI" [model Pa 11] else [model H 4]).],
    [Ferklearrings fan kiezers dat hja de list stypje, want de list komt net yn oanmerking foar de ûntheffing fan dy ferplichtings (#if input.election_type == "KCNI" [model Pa 11] else [model H 4]).],
  )
]
#checkbox(checked: true)[
  #trans(
    "Een verklaring van iedere op de lijst voorkomende kandidaat dat hij instemt met zijn kandidaatstelling op de lijst (model H 9).",
    "In ferklearring fan alle op de list foarkommende kandidaten dat se ynstimme mei harren kandidaatstelling op de list (model H 9).",
  )
]
#checkbox(checked: true)[
  #trans(
    "Een kopie van een geldig identiteitsbewijs van iedere kandidaat die géén zitting heeft in het orgaan waarvoor de verkiezing wordt gehouden.",
    "In kopy fan in jildich identiteitsbewiis fan alle kandidaten dy’t gjin sit hawwe yn it orgaan dêr’t de ferkiezing foar hâlden wurdt.",
  )
]
#checkbox(checked: not input.previously_seated)[
  #trans(
    "Een betalingsbewijs van de waarborgsom, want de lijst komt niet in aanmerking voor de ontheffing van deze verplichting (model H 12).",
    "In betellingsbewiis fan de boarchsom, want de list komt net yn oanmerking foar de ûntheffing fan dy ferplichting (model H 12).",
  )
]
#if input.election_type != "EK" [
  #checkbox(checked: true)[
    #trans(
      "Een verklaring van voorgenomen vestiging van iedere op de lijst voorkomende kandidaat die niet woonachtig is in het gebied waarop de verkiezing betrekking heeft (alleen bij een verkiezing van provinciale staten, het algemeen bestuur van een waterschap, een gemeenteraad, de eilandsraden van de openbare lichamen Bonaire, Saba of Sint Eustatius en de kiescolleges van de openbare lichamen).",
      "In ferklearring fan foarnommen fêstiging foar alle op de list foarkommende kandidaten dy’t net wenjend binne yn it gebiet dêr’t de ferkiezing op slacht (allinnich by in ferkiezing fan provinsjale steaten, it algemien bestjoer fan in wetterskip, in gemeenteried, de eilânrieden fan it iepenbiere lichem Bonêre, Saba of Sint Eustaasjus en de kieskolleezjes fan it iepenbiere lichem).",
    )
  ]
  #checkbox(checked: true)[
    #trans(
      "Een verklaring van voorgenomen vestiging buiten Nederland van iedere op de lijst voorkomende kandidaat die woonachtig is in Nederland (alleen bij een verkiezing van het kiescollege voor niet-ingezetenen).",
      "In ferklearring fan foarnommen fêstiging bûten Nederlân fan elke op de list foarkommende kandidaat dy’t yn Nederlân wennet (allinnich by in ferkiezing fan it kieskolleezje foar net-ynwenners).",
    )
  ]
  #checkbox(checked: true)[
    #trans(
      "Een verklaring van iedere op de lijst voorkomende kandidaat dat hij niet in een andere lidstaat kandidaat zal zijn voor het Europees Parlement (model Y 13).",
      "In ferklearring fan alle op de list foarkommende kandidaten dat se foar it Europeeske Parlemint net yn in oare lidsteat kandidaat wêze sille (model Y 13).",
    )
  ]
  #checkbox(checked: true)[
    #trans(
      "Een verklaring van kandidaten die onderdaan zijn van een andere lidstaat, dat zij in die lidstaat niet zijn uitgesloten van het recht om gekozen te worden voor de verkiezingen van het Europees Parlement (model Y 35).",
      "In ferklearring fan kandidaten dy’t ûnderdaan binne fan in oare lidsteat, dat sy yn dy lidsteat net útsletten binne fan it rjocht om keazen te wurden foar de ferkiezings fan it Europeeske Parlemint (model Y 35).",
    )
  ]
]

= #trans("Ondertekening door de inleveraar", "Undertekening troch dejinge dy’t ynleveret")
#let submitter = input.list_submitter
#label_table(values: (
  (trans("Naam en voorletters", "Namme en foarletters"), [#submitter.last_name, #submitter.initials]),
  (
    trans("Postadres, postcode en plaats", "Postadres, postkoade en plak"),
    [#submitter.postal_address.street_address, #submitter.postal_address.postal_code #submitter.postal_address.locality],
  ),
  (trans("Datum", "Datum"), fill_in()),
  (trans("Handtekening", "Hantekening"), fill_in(height: 4em)),
))
