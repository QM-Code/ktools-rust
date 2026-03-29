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

- The implementation still needs a deliberate parity audit against the full
  C++ contract for selectors, logging behavior, and CLI integration.
- Demo validation is strong now, but the Cargo bin/demo layout still deserves a
  deliberate readability pass so each demo entity is easy to follow.
- Docs and manifests should keep the demo-entity story explicit.
- Any per-demo support modules should stay clearly owned by the demo entity
  they serve rather than drifting into a hidden shared layer.

## Work Plan

1. Re-audit parity with C++.
- Verify channel registration, duplicate color merges, selector parsing,
  unmatched-selector handling, output options, `trace_changed(...)`, and
  `make_inline_parser(...)` behavior against the C++ contract.
- Add focused tests for any reference behavior that is still implicit.

2. Revisit demo/bin readability.
- Review whether the current `src/src/bin/` structure tells a clear story about
  bootstrap, SDK, and executable demos.
- Keep support code obviously owned by the specific demo entity it serves.

3. Treat demos as contract checks.
- Re-run and review `demo/bootstrap`, `demo/sdk/{alpha,beta,gamma}`, and
  `demo/exe/{core,omega}` as part of the project, not just the library crate.
- Make sure README text and demo wiring still tell the same story.

4. Keep the repo layout understandable.
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

- Demo entity ownership is obvious in the Cargo/bin layout.
- Tests and demos cover the reference behavior directly.
