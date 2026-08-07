# module_dependency_cycles

A [Dylint](https://github.com/trailofbits/dylint) library that rejects cyclic
dependencies between components, a component being a top-level directory under
`src/` (files sitting directly in `src/` form the `root` component). Cycles are
of any length: `A -> B -> A` and `A -> B -> C -> A` are both rejected. One error
is emitted per group of components that all reach each other, naming the cheapest
edge in the group and listing the dependencies that make it up. Breaking that one
edge is the cheapest way into the cycle; a group that needs more than one edge
removed reports again on the next run.

Every dependency between components is also reported as a note, cyclic or not,
so a run shows the shape of the graph even when it rejects nothing:

```
note: component dependencies in `eks`:
  root -> src/core (3 dependencies)
  src/api -> src/core (12 dependencies, cyclic)
  src/core -> src/api (1 dependency, cyclic)
```

Dependencies come from resolved paths, so a call reached through the type of a
receiver (`value.method()`) does not count, and neither do references produced
by macro expansion. Test code is skipped: `#[cfg(test)]` blocks (the lint bails
out on a test-mode compilation), `EXCLUDED_DIRECTORIES` or files with
`test` in the filename are excluded.

## Install the tooling

`bin/setup` installs `cargo-dylint` and `dylint-link`, so there is nothing to do
by hand. To install them separately:

```sh
cargo install --locked cargo-dylint dylint-link --version 6.0.3
```

The version must match `dylint_linting` in `Cargo.toml`, both here and in
`development/setup.yml`. `dylint-link` is a linker wrapper, needed to build the
library itself.

## Run the lint

From the repository root:

```sh
cargo dylint --all
```

`--all` loads every library listed in `[workspace.metadata.dylint]` in
`dylint.toml` at the repository root, which is this one. The command exits
non-zero when a cycle is found, since the lint level is `Deny`.

The first run builds the lint library and then type-checks the crate with the
pinned nightly, which takes a few minutes. Builds land in `target/dylint`, a
separate directory from the normal `target`, so this does not invalidate
anything `cargo check` or `cargo clippy` has cached.

`bin/check --full` runs this too. It is left out of a plain `bin/check` because
of the run time.

Useful variations:

```sh
cargo dylint list                # show the discovered library and its toolchain
cargo dylint --all --workspace   # lint every workspace member, not just eks
cargo dylint --all --no-build    # reuse the library binary as it was last built
```

## Work on the lint

The library is deliberately not a member of the root workspace: it needs the
pinned nightly, `rustc_private`, and a `cdylib` crate type. `cargo dylint`
builds it as part of a run, so a plain `cargo dylint --all` from the repository
root is enough while iterating. To compile it on its own, run `cargo build
--release` in this directory.

