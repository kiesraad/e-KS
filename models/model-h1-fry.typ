#import "layout.typ": checkbox, column_table, conf, date, enumerated_table, fill_in, label_table, mono

#let input = json("./input.json")

#show: doc => conf(
  doc,
  "Model H 1",
  "Kandidatelist",
  "Mei dit formulier stelle jo, as dejinge dy’t de kandidatelist ynleveret, kandidaten ferkiesber foar in ferkiezing.",
  page-label: (n, m) => [Side #n fan #m],
  input,
)


= Ferkiezing
It giet om de ferkiezing fan: *#input.election_name*


= Kiesrûnten
De kandidatelist wurdt ynlevere foar:
#if input.electoral_districts.tag == "All" {
  [*alle kiesrûnten*]
} else {
  block(above: 1em, list(tight: true, ..input.electoral_districts.districts))
}


= De politike groepearring
De politike groepearring dêr’t jo de kandidatelist fan stypje: *#input.designation*


= Kandidaten op de list
#column_table(
  columns: (auto, 1fr, 1fr, 1fr, 1fr),
  headers: ("", "namme", "foarletters", "bertedatum", "wenplak"),
  values: input.candidates.map(c => (
    [#c.position],
    c.last_name,
    c.initials,
    date(c.date_of_birth),
    c.locality,
  )),
)


= Ferfanger(s) foar it ferhelpen fan fersommen
#if input.substitute_submitter.len() == 0 {
  [_geen_]
} else {
  enumerated_table(
    columns: (1fr, 1fr, 1fr, 0.75fr, 1.5fr),
    headers: ("namme", "foarletters", "postadres", "postkoade", "plak"),
    values: input.substitute_submitter.map(s => (
      s.last_name,
      s.initials,
      s.address_line_1,
      mono(s.postal_code),
      s.locality,
    )),
  )
}


= Yn te leverjen by de kandidatelist
Ik bin ferplichte de neikommende taheakke by de kandidatelist yn te leverjen:

#checkbox(checked: true)[
  In ferklearring fan de lêsthawwer(s) fan de politike groepearring(s) dêr’t my tastimming mei jûn wurdt om de oantsjutting boppe de kandidatelist te pleatsen, want ik haw in oantsjutting boppe de list pleatst (model H 3-1 of H 3-2).
]
#checkbox(checked: not input.previously_seated)[
  Ferklearrings fan kiezers dat hja de list stypje, want de list komt net yn oanmerking foar de ûntheffing fan dy ferplichtings (#if input.election_type == "KCNI" [model Pa 11] else [model H 4]).
]
#checkbox(checked: true)[
  In ferklearring fan alle op de list foarkommende kandidaten dat se ynstimme mei harren kandidaatstelling op de list (model H 9).
]
#checkbox(checked: true)[
  In kopy fan in jildich identiteitsbewiis fan alle kandidaten dy’t gjin sit hawwe yn it orgaan dêr’t de ferkiezing foar hâlden wurdt.
]
#checkbox(checked: not input.previously_seated)[
  In betellingsbewiis fan de boarchsom, want de list komt net yn oanmerking foar de ûntheffing fan dy ferplichting (model H 12).
]
#if input.election_type != "EK" [
  #checkbox(checked: true)[
    In ferklearring fan foarnommen fêstiging foar alle op de list foarkommende kandidaten dy’t net wenjend binne yn it gebiet dêr’t de ferkiezing op slacht (allinnich by in ferkiezing fan provinsjale steaten, it algemien bestjoer fan in wetterskip, in gemeenteried, de eilânrieden fan it iepenbiere lichem Bonêre, Saba of Sint Eustaasjus en de kieskolleezjes fan it iepenbiere lichem).
  ]
  #checkbox(checked: true)[
    In ferklearring fan foarnommen fêstiging bûten Nederlân fan elke op de list foarkommende kandidaat dy’t yn Nederlân wennet (allinnich by in ferkiezing fan it kieskolleezje foar net-ynwenners).
  ]
  #checkbox(checked: true)[
    In ferklearring fan alle op de list foarkommende kandidaten dat se foar it Europeeske Parlemint net yn in oare lidsteat kandidaat wêze sille (model Y 13).
  ]
  #checkbox(checked: true)[
    In ferklearring fan kandidaten dy’t ûnderdaan binne fan in oare lidsteat, dat sy yn dy lidsteat net útsletten binne fan it rjocht om keazen te wurden foar de ferkiezings fan it Europeeske Parlemint (model Y 35).
  ]
]


= Undertekening troch dejinge dy’t ynleveret
#let submitter = input.list_submitter
#label_table(values: (
  ("Namme en foarletters", [#submitter.last_name, #submitter.initials]),
  ("Postadres, postkoade en plak", [#submitter.address_line_1#linebreak()#submitter.address_line_2]),
  ("Datum", fill_in()),
  ("Hantekening", fill_in(height: 4em)),
))
