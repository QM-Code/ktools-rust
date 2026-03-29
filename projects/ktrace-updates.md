# Rust ktrace Project

## Mission

Keep `ktools-rust/ktrace/` structurally simple and fully parity-audited while
preserving Rust-idiomatic APIs and error handling.

## Required Reading

- `../ktools/AGENTS.md`
- `AGENTS.md`
- `ktrace/AGENTS.md`
- `ktrace/README.md`
- `ktrace/docs/api.md`
- `ktrace/docs/selectors.md`
- `../ktools-cpp/ktrace/README.md`
- `../ktools-cpp/ktrace/include/ktrace.hpp`
- `../ktools-cpp/ktrace/src/ktrace/cli.cpp`
- `../ktools-cpp/ktrace/cmake/tests/ktrace_channel_semantics_test.cpp`
- `../ktools-cpp/ktrace/cmake/tests/ktrace_format_api_test.cpp`
- `../ktools-cpp/ktrace/cmake/tests/ktrace_log_api_test.cpp`

## Current Gaps

- A large amount of generated output is still tracked under `ktrace/build/`,
  `ktrace/src/target/`, and `ktrace/.cargo-home/`.
- Demo support code currently lives in `src/src/demo/`, which blurs the line
  between the library crate and demo entities.
- The implementation still needs a deliberate parity audit against the full
  C++ contract for selectors, logging behavior, and CLI integration.
- Demo validation should remain first-class, not secondary to library tests.

## Work Plan

1. Clean repo hygiene aggressively.
- Remove tracked generated artifacts from staged build directories, Cargo
  target output, and local Cargo cache state.
- Tighten ignore rules so generated output does not return.

2. Get shared demo code out of the library crate.
- Remove the current shared demo helper layer under `src/src/demo/`.
- Make the bootstrap, SDK, and executable demo binaries readable as separate
  entities that compose together.
- Keep demo support logic with the owning demo binary or crate instead of a
  central shared helper module.

3. Re-audit parity with C++.
- Verify channel registration, duplicate color merges, selector parsing,
  unmatched-selector handling, output options, `trace_changed(...)`, and
  `make_inline_parser(...)` behavior against the C++ contract.
- Add focused tests for any reference behavior that is still implicit.

4. Treat demos as contract checks.
- Re-run and review `demo/bootstrap`, `demo/sdk/{alpha,beta,gamma}`, and
  `demo/exe/{core,omega}` as part of the project, not just the library crate.
- Make sure README text and demo wiring still tell the same story.

5. Keep the repo layout understandable.
- Preserve Rust-idiomatic error handling and API usage.
- Avoid replacing one awkward layout with another.

## Constraints

- Preserve Rust-idiomatic APIs and error handling.
- Do not disturb the demo entity structure unless the replacement is clearly
  better.
- Prefer real clarity improvements over cosmetic renaming.

## Validation

- `cd ktools-rust/ktrace && kbuild --build-latest`
- `cd ktools-rust/ktrace/src && cargo test`
- Run the staged demo commands listed in `ktools-rust/ktrace/README.md`

## Done When

- Generated output no longer dominates the repo.
- Shared demo code is gone from the library crate.
- Tests and demos cover the reference behavior directly.
