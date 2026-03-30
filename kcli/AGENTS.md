# kcli-rust

Assume these have already been read:

- `../../ktools/AGENTS.md`
- `../AGENTS.md`

`ktools-rust/kcli/` is the Rust implementation component for `kcli`.

## What This Component Owns

- Rust API and parsing behavior for `kcli`
- Rust demo packages and tests
- workspace `kbuild` integration for this component

## Local Bootstrap

Read:

- `README.md`
- `Cargo.toml`
- `src/*`
- `tests/*`
- `demo/*/*/Cargo.toml`
- `demo/*/*/src/*`

After a coherent batch of changes in `ktools-rust/kcli/`, return to the
`ktools-rust/` workspace root and run `kbuild --git-sync "<message>"`.
