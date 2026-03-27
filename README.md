# ktools-rust

`ktools-rust/` is the Rust workspace for the broader ktools ecosystem.

It is the root entrypoint for Rust implementations of the ktools libraries.

## Current Contents

This workspace currently contains:

- `kcli/`
- `ktrace/`
- `kbuild/` local Rust-specific `kbuild` copy

## Build Model

Use the relevant child repo when building or testing a specific Rust implementation.

This workspace carries a Rust-local `kbuild` copy under `kbuild/` plus executable `kbuild.py` wrappers at the workspace root and child repo roots. That local copy adds Cargo support without modifying the C++-oriented `kbuild` tree.

## Where To Go Next

For concrete Rust API and implementation details, use the docs in the relevant child repo.

Current implementation:

- [kcli](kcli)
- [ktrace](ktrace)
- [kbuild docs](kbuild/README.md)
