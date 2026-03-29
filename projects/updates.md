# Rust Updates

## Mission

Keep `ktools-rust/` structurally simple and fully parity-audited while
preserving Rust-idiomatic APIs and error handling across both `kcli` and
`ktrace`.

## Required Reading

- `../ktools/AGENTS.md`
- `AGENTS.md`
- `README.md`
- `kcli/AGENTS.md`
- `kcli/README.md`
- `ktrace/AGENTS.md`
- `ktrace/README.md`
- `ktrace/docs/api.md`
- `ktrace/docs/selectors.md`
- `../ktools-cpp/kcli/README.md`
- `../ktools-cpp/kcli/docs/behavior.md`
- `../ktools-cpp/kcli/cmake/tests/kcli_api_cases.cpp`
- `../ktools-cpp/ktrace/README.md`
- `../ktools-cpp/ktrace/include/ktrace.hpp`
- `../ktools-cpp/ktrace/src/ktrace/cli.cpp`
- `../ktools-cpp/ktrace/cmake/tests/ktrace_channel_semantics_test.cpp`
- `../ktools-cpp/ktrace/cmake/tests/ktrace_format_api_test.cpp`
- `../ktools-cpp/ktrace/cmake/tests/ktrace_log_api_test.cpp`

## kcli Focus

- Review whether `kcli/src/process/plan.rs` should be split further.
- Consider splitting `kcli/tests/api.rs` by concern if that would make
  failures easier to localize.
- Re-audit aliases, bare inline roots, required and optional values, help
  output, double-dash rejection, and error behavior against the C++ contract.
- Treat the demo crates as contract checks, not just support material.

## ktrace Focus

- Re-audit selector parsing, duplicate color merges, unmatched-selector
  handling, output options, `trace_changed(...)`, and
  `make_inline_parser(...)` behavior against the C++ contract.
- Revisit the Cargo bin and demo layout so bootstrap, SDK, and executable
  entities are easy to follow.
- Keep any per-demo support code obviously owned by the entity it serves.

## Cross-Cutting Rules

- Preserve Rust-idiomatic API usage and error handling.
- Do not replace the current demo entity structure with a hidden shared layer.
- Keep `target/`, `.cargo-home/`, and other generated output out of version
  control.

## Validation

- `cd ktools-rust/kcli && kbuild --build-latest`
- `cd ktools-rust/kcli && cargo test --manifest-path Cargo.toml`
- `cd ktools-rust/kcli && cargo test --manifest-path demo/exe/core/Cargo.toml`
- `cd ktools-rust/kcli && cargo test --manifest-path demo/exe/omega/Cargo.toml`
- `cd ktools-rust/ktrace && kbuild --build-latest`
- `cd ktools-rust/ktrace/src && cargo test`
- Run the staged demo commands listed in each repo README

## Done When

- `kcli` and `ktrace` both cover the C++ contract directly.
- Demo entity ownership is obvious in the Cargo layout.
- The repo remains unsurprising to a Rust reviewer.
