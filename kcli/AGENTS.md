# kcli-rust

Assume these have already been read:

- `../../ktools/AGENTS.md`
- `../AGENTS.md`

`ktools-rust/kcli/` is the Rust implementation repo for `kcli`.

## What This Repo Owns

- Rust API and parsing behavior for `kcli`
- Rust demo packages and tests
- workspace `kbuild` integration for this repo

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
