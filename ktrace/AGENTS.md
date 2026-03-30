# ktrace-rust

Assume these have already been read:

- `../../ktools/AGENTS.md`
- `../AGENTS.md`

`ktools-rust/ktrace/` is the Rust implementation component for `ktrace`.

## What This Component Owns

- Rust API and runtime behavior for `ktrace`
- Rust-side `kcli` integration for trace CLI controls
- Rust tests and demo binaries
- workspace `kbuild` integration for this component

## Local Bootstrap

Read:

- `README.md`
- `src/Cargo.toml`
- `src/src/*`
- `src/tests/*`

## Build And Test Expectations

- Use `kbuild --build-latest` from this component root for normal builds.
- Use `cargo test` inside `src/` for crate-level iteration when needed.
- Keep behavior aligned with the cross-language `ktrace` model unless there is a strong Rust-specific reason not to.

After a coherent batch of changes in `ktools-rust/ktrace/`, return to the
`ktools-rust/` workspace root and run `kbuild --git-sync "<message>"`.
