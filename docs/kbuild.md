# `kbuild` In `ktools-rust`

The Rust workspace uses the shared `kbuild` command model for workspace build
orchestration.

## Current Status

- the checked-out workspace does not currently contain a separate `kbuild/`
  implementation directory
- the expected entrypoint is the shared `kbuild` command on `PATH`
- Rust-specific build behavior should still align with the shared command
  surface
