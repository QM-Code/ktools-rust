# Rust Build Cleanup Project

## Mission

Add a Rust-specific residual checker to `kbuild`, then make the Rust workspace
stop generating Cargo artifacts outside `build/`.

This task spans both `ktools-rust/` and the sibling shared build repo
`../kbuild/`.

## Required Reading

- `../ktools/AGENTS.md`
- `AGENTS.md`
- `README.md`
- `../kbuild/AGENTS.md`
- `../kbuild/README.md`
- `../kbuild/libs/kbuild/residual_ops.py`
- `../kbuild/libs/kbuild/backend_ops.py`
- `../kbuild/libs/kbuild/cargo_backend.py`
- `../kbuild/tests/test_java_residuals.py`
- `kcli/AGENTS.md`
- `kcli/README.md`
- `ktrace/AGENTS.md`
- `ktrace/README.md`

## Current Gaps

- `kbuild` does not yet have a Rust backend residual checker.
- The Rust workspace currently carries Cargo-generated output outside
  `build/`, notably `target/` and local Cargo cache/home state.
- The build flow needs to be corrected so Cargo writes staged output under
  `build/` instead of the source tree.

## Work Plan

1. Add the Rust residual checker in `kbuild`.
- Follow the Java checker structure, but make it Rust-specific.
- Detect real Cargo build residuals outside `build/`, such as `target/`,
  source-tree Cargo cache state, or equivalent generated output.
- Keep the checker narrow and tied to actual Cargo build behavior.

2. Add focused `kbuild` tests.
- Add tests for build refusal and `--git-sync` refusal when Rust build
  residuals appear outside `build/`.
- Add a positive case showing that staged output inside `build/` is allowed.

3. Audit the actual Rust workspace build flow.
- Build `kcli/` and `ktrace/` through normal `kbuild` entrypoints.
- Identify where Cargo is still writing to `target/` or other source-tree
  locations.
- Fix the build flow so staged paths under `build/` are used consistently.

4. Clean up real residuals.
- Remove existing Cargo build/cache artifacts from the source tree where they
  do not belong.
- Tighten ignore rules only after the actual generation path is fixed.

5. Keep docs aligned.
- Update `kbuild` docs if the checker needs backend-specific mention.
- Update local docs if they currently normalize source-tree Cargo output.

## Constraints

- Do not accept source-tree `target/` or equivalent Cargo output as normal.
- Prefer redirecting Cargo correctly over adding ignore-only band-aids.
- Keep the checker and tests precise.

## Validation

- Run the new Rust residual tests in `../kbuild`
- `cd ktools-rust && kbuild --batch --build-latest`
- `cd ktools-rust/kcli/src && cargo test`
- `cd ktools-rust/ktrace/src && cargo test`
- Confirm the workspace stays free of Cargo-generated output outside `build/`

## Done When

- `kbuild` rejects Rust/Cargo residuals outside `build/`.
- The Rust workspace no longer generates those residuals in normal use.
- Cargo output is staged under `build/` instead of leaking into the repo tree.
