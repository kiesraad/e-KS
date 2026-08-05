# module_dependency_cycles

A [Dylint](https://github.com/trailofbits/dylint) library that rejects cyclic
dependencies between components, a component being a top-level directory under
`src/` (files sitting directly in `src/` form the `Remainder` component). One
error is emitted per cyclic pair, naming the direction that is cheaper to
remove and listing the dependencies that make up that direction.

Dependencies come from resolved paths, so a call reached through the type of a
receiver (`value.method()`) does not count, and neither do references produced
by macro expansion. Test code is skipped: `#[cfg(test)]` blocks (the lint bails
out on a test-mode compilation), `EXCLUDED_DIRECTORIES` or files with
`test` in the filename are excluded.

## Install the tooling

```sh
cargo install cargo-dylint dylint-link --version 6.0.3
```

The version must match `dylint_linting` in `Cargo.toml`. `dylint-link` is a
linker wrapper, needed to build the library itself.

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

