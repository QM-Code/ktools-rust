# Rust kcli Project

## Mission

Keep `ktools-rust/kcli/` structurally simple and fully parity-audited while
preserving Rust-idiomatic API and error handling.

## Required Reading

- `../ktools/AGENTS.md`
- `AGENTS.md`
- `kcli/AGENTS.md`
- `kcli/README.md`
- `../ktools-cpp/kcli/README.md`
- `../ktools-cpp/kcli/docs/behavior.md`
- `../ktools-cpp/kcli/cmake/tests/kcli_api_cases.cpp`

## Current Gaps

- `kcli/src/process/plan.rs` is still the largest parser module.
- API behavior coverage is still concentrated in a single `tests/api.rs` file.
- Demo crates and their validation flow should be treated as part of the
  contract, not just supporting examples.
- Docs and manifests should be rechecked so the flattened layout stays easy to
  understand.

## Work Plan

1. Revisit the largest parser module.
- Review whether `src/process/plan.rs` should be split further into smaller,
  coherent pieces.
- Keep public API exposure centralized and easy to follow.

2. Improve test discoverability.
- Consider splitting `tests/api.rs` by concern if that would make failures
  easier to localize.
- Preserve the current strong behavior coverage while making it easier to scan.

3. Re-audit parity with C++.
- Verify aliases, bare inline roots, required/optional values, help output,
  double-dash rejection, and error behavior against the C++ docs/tests.
- Add focused tests for any reference behavior that is still implicit.

4. Treat demo crates as contract checks.
- Re-run and review `demo/bootstrap`, `demo/sdk/{alpha,beta,gamma}`, and
  `demo/exe/{core,omega}` as part of the project, not just the library crate.
- Make sure README text and demo manifests still tell the same story.

5. Keep hygiene rules tight.
- Ensure `target/`, `.cargo-home/`, and staged build directories remain out of
  version control.
- Avoid reintroducing layout confusion after the flattening work.

## Constraints

- Preserve Rust-idiomatic API usage and error handling.
- Do not disturb the demo crate structure unless the replacement is clearly
  better.
- Prefer real clarity improvements over cosmetic renaming.

## Validation

- `cd ktools-rust/kcli && kbuild --build-latest`
- `cd ktools-rust/kcli && cargo test --manifest-path Cargo.toml`
- `cd ktools-rust/kcli && cargo test --manifest-path demo/exe/core/Cargo.toml`
- `cd ktools-rust/kcli && cargo test --manifest-path demo/exe/omega/Cargo.toml`

## Done When

- The parser internals are easy to navigate.
- Tests and demo crates cover the reference behavior directly.
- The repo layout remains unsurprising to a Rust reviewer.
