We use [Typst](https://typst.app/) to render PDFs.


- Typst can be installed using cargo: `cargo install --locked typst-cli`
- Compile a file once: `typst compile file.typ`
- Compile a file on every change: `typst watch file.typ`

The `inputs` folder contains various example JSON inputs for the templates.
- TIP: Create a symlink to the example input for the template you're developing:
  -  For example, for the H1 model use: `ln -s "$(pwd)/models/example-inputs/model-h1-example-1.json" ./models/input.json`
