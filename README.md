# ktools-rust

`ktools-rust/` is the Rust workspace for the broader ktools ecosystem.

It is the root entrypoint for Rust implementations of the ktools libraries.

## Current Contents

This workspace currently contains:

- `kcli/`
- `ktrace/`

## Build Model

Use the relevant child repo when building or testing a specific Rust implementation.

The shared `kbuild` implementation lives in the sibling [`../kbuild/`](../kbuild/)
repo. Use `kbuild` from `PATH` when available, or invoke the shared script
directly:

```bash
python3 ../kbuild/kbuild.py --help
```

Each child repo also supports direct Cargo workflows. Repo-local Cargo config
routes those outputs under this workspace's `build/` tree instead of leaving
`target/` directories in the source repos.

## Where To Go Next

For concrete Rust API and implementation details, use the docs in the relevant child repo.

Current implementation:

- [kcli](kcli)
- [ktrace](ktrace)
