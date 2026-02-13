#let conf(doc, model, name, explanation, input) = [
  #set text(
    lang: "nl",
    region: "nl",
    font: "DM Sans",
    size: 9pt,
  )

  #let footer = grid(
    columns: 1fr,
    gutter: .5em,
    align(left, if "sha_hash" in input [ SHA-256 hash code:#h(.5em) #input.sha_hash ]),
    context grid(
      columns: (1fr, auto),
      [#input.timestamp], [Pagina #counter(page).display((n, m) => [#n van #m], both: true)],
    ),
  )

  #set page(
    paper: "a4",
    margin: (x: 1.5cm, y: 2.5cm),
    header: align(right)[#model - #name],
    footer: footer,
  )

  #show heading.where(level: 2): set block(above: 3em, below: 1em)

  #grid(
    columns: 1fr,
    gutter: 1.33em,
    grid.hline(stroke: 1pt),
    v(0.5em),
    text(size: 1.5em, weight: "semibold", model),
    text(size: 2em, weight: "semibold", name),
    text(explanation),
    v(0.5em),
    grid.hline(stroke: 1pt),
  )

  #doc
]

/// Table with numbers in the first column
#let enumerated_table(columns: (), headers: (), values: ()) = [
  #table(
    stroke: none,
    inset: 0.75em,
    align: horizon,
    columns: (auto, ..columns),
    table.header([], ..headers.map(value => { text(style: "italic", value) })),
    ..values.enumerate().map(((i, value)) => (str(i + 1), value)).flatten(),
  )
]

/// Line with space to fill in later
#let fill_in = box(width: 100%, height: 1.5em, stroke: (bottom: 1pt + black), inset: 0pt)[]

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
