# ktools-rust

Assume `../ktools/AGENTS.md` has already been read.

`ktools-rust/` is the Rust workspace for the ktools ecosystem.

## What This Level Owns

This workspace owns Rust-specific concerns such as:

- crate/module layout
- Rust build and test flow
- Rust-specific API naming and integration patterns
- coordination across Rust tool implementations when more than one repo is present

Cross-language conceptual definitions belong at the overview/spec level, not here.

## Current Scope

This workspace currently contains:

- `kcli/`
- `ktrace/`
- `kbuild/` (Rust-local copy for Cargo integration)

## Guidance For Agents

1. First determine whether the task belongs at the workspace root or inside a specific implementation repo.
2. Prefer making changes in the narrowest repo that actually owns the behavior.
3. Use the root workspace only for Rust-workspace-wide concerns such as root docs or cross-repo coordination.
4. Read the relevant child repo `AGENTS.md` and `README.md` files before changing code in that repo.
5. Prefer the local Rust workspace `kbuild.py` and `kbuild/` copy when the task depends on Rust-specific build behavior.
