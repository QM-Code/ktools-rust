# ktrace-rust

Assume these have already been read:

- `../../ktools/AGENTS.md`
- `../AGENTS.md`

`ktools-rust/ktrace/` is the Rust implementation repo for `ktrace`.

## What This Repo Owns

- Rust API and runtime behavior for `ktrace`
- Rust-side `kcli` integration for trace CLI controls
- Rust tests and demo binaries
- workspace `kbuild` integration for this repo

## Local Bootstrap

Read:

- `README.md`
- `src/Cargo.toml`
- `src/src/*`
- `src/tests/*`

## Build And Test Expectations

- Use `kbuild --build-latest` from this repo root for normal builds.
- Use `cargo test` inside `src/` for crate-level iteration when needed.
- Keep behavior aligned with the cross-language `ktrace` model unless there is a strong Rust-specific reason not to.
