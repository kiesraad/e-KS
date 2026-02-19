#let conf(
  title: none,
  authors: (),
  abstract: [],
  doc,
) = {
  set page(
    "a4",
    margin: auto,
    header: align(
      right + horizon,
      title
    ),
    numbering: "1",
    columns: 2,
  )
  set par(justify: true)
  set text(font: "Bagnard", 11pt)

  place(
    top + center,
    float: true,
    scope: "parent",
    clearance: 2em,
    {
      text(
        17pt,
        weight: "bold",
        title,
      )

      let count = authors.len()
      let ncols = calc.min(count, 3)
      grid(
        columns: (1fr,) * ncols,
        row-gutter: 24pt,
        ..authors.map(author => [
          #author.name \
          #author.affiliation \
          #link("mailto:" + author.email)
        ]),
      )
      par(justify: false)[
        *Abstract* \
        #abstract
      ]
    }
  )

  doc
}

#show: conf.with(
  title: [
    A Typst web service example
  ],
  authors: (
    (
      name: "Marlon",
      affiliation: "Tweede golf",
      email: "marlon@example.com",
    ),
  ),
  abstract: lorem(80),
)
#let input = json("input.json")

= Hello [#input.name]
#lorem(60)

#for item in input.list [
  + #item
]

== Motivation
#lorem(120)

== Problem Statement
#lorem(60)

= Related Work
#lorem(20)
