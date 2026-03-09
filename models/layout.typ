#let mono(content) = text(font: "Geist Mono", content)

#let conf(doc, model, name, explanation, page-label: (n, m) => [Pagina #n van #m], input) = [
  #set text(
    lang: "nl",
    region: "nl",
    font: "DM Sans",
    size: 9pt,
  )

  #let footer = grid(
    columns: 1fr,
    gutter: .75em,
    context grid(
      columns: (1fr, auto),
      [#datetime(..input.timestamp).display("[day]-[month]-[year] [hour repr:24]:[minute]:[second]")],
      counter(page).display(page-label, both: true),
    ),
    align(left, if "sha_hash" in input [ SHA-256:#h(.5em) #mono(input.sha_hash) ]),
  )

  #set page(
    paper: "a4",
    margin: (x: 1.5cm, y: 2.5cm),
    header: align(right)[#model - #name],
    footer: footer,
  )

  #set heading(numbering: "1.")
  #show heading.where(level: 1): set block(above: 3em, below: 1em)

  #set table(stroke: none, inset: 0.75em, align: horizon)

  #grid(
    columns: 1fr,
    gutter: 1.33em,
    grid.hline(stroke: 1pt),
    v(0.5em),
    text(size: 1.5em, weight: "bold", model),
    text(size: 2em, weight: "bold", name),
    text(explanation),
    v(0.5em),
    grid.hline(stroke: 1pt),
  )

  #doc
]

#let column_table(columns: (), headers: (), values: ()) = {
  assert.eq(
    columns.len(),
    headers.len(),
    message: "columns.len() ("
      + repr(columns.len())
      + ") != headers.len() ("
      + repr(headers.len())
      + ")\ncolumns="
      + repr(columns)
      + "\nheaders="
      + repr(headers),
  )
  assert.eq(
    columns.len(),
    values.at(0).len(),
    message: "columns.len() ("
      + repr(columns.len())
      + ") != values[0].len() ("
      + repr(values.at(0).len())
      + ")\ncolumns="
      + repr(columns)
      + "\nvalues[0]="
      + repr(values.at(0)),
  )

  table(
    columns: columns,
    table.header(..headers.map(value => { text(style: "italic", value) })),
    ..values.flatten(),
  )
}

/// Table with numbers in the first column
#let enumerated_table(columns: (), headers: (), values: ()) = column_table(
  columns: (auto, ..columns),
  headers: ([], ..headers),
  values: values.enumerate().map(((i, value)) => (str(i + 1), ..value)),
)

/// Table with two columns, with labels on the left
#let label_table(values: ()) = table(
  columns: (auto, 1fr),
  ..values.flatten(),
)

/// Line with space to fill in later
#let fill_in(height: 2em) = box(width: 100%, height: height, stroke: (bottom: 1pt + black), inset: 0pt)[]

#let date(date) = mono(datetime(..date).display("[day]-[month]-[year]"))

/// Display a checkmark for usage in a checkbox
#let checkmark() = {
  box(width: 100%, height: 100%, clip: false, curve(
    stroke: (thickness: 2pt, cap: "round", join: "miter", paint: white),
    curve.move((0%, 50%)),
    curve.line((40%, 90%)),
    curve.line((90%, 0%)),
  ))
}

/// Display a checkbox, optionally already checked when the `checked` parameter is set to `true`
#let checkbox(checked: none, content) = {
  let has_content = content != none and content != ""
  let size = 9pt

  grid(
    columns: if has_content { (14pt, 6pt, auto) } else { (size) },
    align: horizon + center,
    box(
      width: size,
      height: size,
      inset: 2.5pt,
      stroke: if checked == none or checked == true { 0.5pt + black } else {
        (thickness: 0.4pt, dash: "densely-dotted", cap: "square")
      },
      clip: true,
      fill: if checked == true { black } else { white },
      if checked == true { checkmark() },
    ),
    if has_content { " " },
    if has_content { align(left, content) },
  )
}
