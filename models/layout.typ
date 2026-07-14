#let mono(content) = text(font: "Geist Mono", content)

#let muted(content) = text(fill: rgb("888888"), content)

#let highlight_colour = rgb("F6F6F6")

#let translator(locale) = (dutch, frisian) => if locale == "fry" { frisian } else { dutch }

#let conf(doc, model, name, explanation, warning: none, input) = [
  #let trans = if "locale" in input { translator(input.locale) } else { translator("nl") }

  #set text(
    lang: "nl",
    region: "nl",
    font: "DM Sans",
    size: 9pt,
  )

  // make all paragraphs sticky to prevent unnecessary page breaks within sections
  #show par: it => block(sticky: true, it)

  #let footer = grid(
    columns: 1fr,
    gutter: .75em,
    context grid(
      columns: (1fr, auto),
      [
        #if "event_id" in input [
          #muted(trans[Versie:][Ferzje:]) #mono[#input.event_id]
          #h(1em)
        ]
        #if "sha_hash" in input [
          #muted[Hash:] #mono(input.sha_hash)
        ]
      ],
      counter(page).display((n, m) => trans([Pagina #n van #m], [Side #n fan #m]), both: true),
    ),
  )

  #set page(
    paper: "a4",
    margin: (x: 1.5cm, y: 2.5cm),
    header: align(right)[#model - #name],
    footer: footer,
  )

  #set heading(numbering: "1.", supplement: none)
  #show heading.where(level: 1): it => {
    // prevent stickiness from sticking to level 1 headers (because all paragraphs are sticky)
    block(sticky: false, spacing: 0pt)[#box(width: 0pt, height: 0pt)[]]
    it
  }
  #show heading.where(level: 1): set block(above: 2em, below: 0.75em)
  #show heading.where(level: 2): set heading(numbering: none)
  #show heading.where(level: 3): set heading(numbering: none)

  #set table(stroke: none, inset: 0.75em, align: horizon)

  #grid(
    columns: 1fr,
    gutter: 1.33em,
    text(size: 1.5em, weight: "bold", model),
    text(size: 2em, weight: "bold", {
      set par(leading: 0.4em)
      name
    }),
    text(explanation),
    if warning != none {
      block(fill: highlight_colour, inset: 1em, width: 100%, warning)
    }
  )

  #doc
]

#let column_table(columns: (), headers: (), values: (), align: horizon) = {
  assert.eq(
    columns.len(),
    headers.len(),
    message: "the number of headers does not match the number of columns",
  )
  if values.len() > 0 {
    assert.eq(
      columns.len(),
      values.at(0).len(),
      message: "the first row of values does not match the number of columns",
    )
  }

  block(breakable: values.len() > 10, table(
    columns: columns,
    align: align,
    rows: 1.45em,
    fill: (_, y) => if calc.odd(y) { highlight_colour },
    table.header(..headers.map(value => { text(style: "italic", value) })),
    ..values.flatten(),
  ))
}
/// Table without alternating row colors and row height that fits content
#let plain_table(columns: (), headers: (), values: ()) = {
  let values = values.flatten()
  block(breakable: values.len() > 10, table(
    columns: columns,
    align: top,
    gutter: 1em,
    inset: 0em,
    table.header(..headers.map(value => { text(style: "italic", size: .9em, value) })),
    ..values,
  ))
}

/// Table with numbers in the first column
#let enumerated_table(columns: (), headers: (), values: ()) = column_table(
  columns: (auto, ..columns),
  headers: ([], ..headers),
  values: values.enumerate().map(((i, value)) => (str(i + 1), ..value)),
)

/// Table with two columns, with labels on the left
#let label_table(values: ()) = block(breakable: false, table(
  columns: (1fr, 2fr),
  ..values.flatten(),
  gutter: 1em,
  inset: 0em
))

/// Line with space to fill in later
#let fill_in(height: 2em, width: 100%) = box(width: width, height: height, stroke: (bottom: 1pt + black), inset: 0pt)[]

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
#let checkbox(checked: true, content) = {
  let has_content = content != none and content != ""
  let size = 9pt

  block(
    sticky: true,
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
    ),
  )
}
