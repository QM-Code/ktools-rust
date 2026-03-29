# Rust kcli Project

## Mission

Bring `ktools-rust/kcli/` structurally up to the C++ reference while preserving
Rust idioms. The current repo is functional, but its workspace/crate layout is
the most awkward non-Swift layout in the stack.

## Required Reading

- `../ktools/AGENTS.md`
- `AGENTS.md`
- `kcli/AGENTS.md`
- `kcli/README.md`
- `../ktools-cpp/kcli/README.md`
- `../ktools-cpp/kcli/docs/behavior.md`
- `../ktools-cpp/kcli/cmake/tests/kcli_api_cases.cpp`

## Current Gaps

- The repo root is a Cargo workspace, but the main crate lives in `kcli/src/`.
- Source files live under `kcli/src/src/`, which is awkward to navigate.
- Tests live under `kcli/src/tests/` instead of a more obvious top-level test
  location.
- `kcli/src/src/process.rs` is very large.
- The repo needs a deliberate parity check against the C++ behavior contract.

## Work Plan

1. Flatten the crate layout.
- Move toward a more normal Rust shape where the main crate lives at the repo
  root or, at minimum, remove the `src/src` awkwardness.
- Make the top-level file layout clearly communicate workspace members versus
  the main library crate.
- Keep demo crates intact unless the workspace layout itself becomes clearer by
  moving them.

2. Split oversized parser implementation files.
- Break `process.rs` into coherent modules if that improves readability.
- Keep public API exposure centralized and easy to follow.
- Preserve the current good separation around normalization/model/backend unless
  a cleaner split is obvious.

3. Improve test discoverability.
- Move or reorganize tests so API tests are found where a Rust reviewer expects
  them.
- Preserve the existing demo crate tests, since those are strong end-to-end
  checks.

4. Audit behavior parity with C++.
- Confirm matching semantics for aliases, bare inline roots, required and
  optional values, help output, and error behavior.
- Add focused tests for any reference behavior that is currently implicit.

5. Review docs and demos after the structural cleanup.
- Keep the bootstrap/sdk/exe demo roles aligned with C++.
- Ensure the README and source layout agree after any workspace restructuring.

## Constraints

- Preserve Rust-idiomatic API usage and error handling.
- Do not break the demo crate structure unless the replacement is clearly
  better.
- Prefer real layout simplification over cosmetic renaming.

## Validation

- `cd ktools-rust/kcli && kbuild --build-latest`
- `cd ktools-rust/kcli && cargo test --manifest-path Cargo.toml`
- `cd ktools-rust/kcli && cargo test --manifest-path demo/exe/core/Cargo.toml`
- `cd ktools-rust/kcli && cargo test --manifest-path demo/exe/omega/Cargo.toml`

## Done When

- The main crate layout no longer feels nested or surprising.
- Oversized parser internals are split into clearer modules.
- A Rust reviewer can compare the repo to C++ without first decoding the
  workspace layout.
