# Rust-Local Cargo Extensions

This `kbuild/` copy is local to `ktools-rust/`. It extends the shared tool with
a Cargo execution path instead of modifying the C++-oriented copy.

## New Config Section

Rust repos can define a top-level `cargo` object:

```json
{
  "cargo": {
    "manifest": "src/Cargo.toml",
    "package": "ktrace",
    "tests": true,
    "sdk": {
      "include": ["Cargo.toml", "Cargo.lock", "README.md", "src", "tests"]
    },
    "demos": {
      "exe/core": { "bin": "core" },
      "exe/omega": { "bin": "omega" }
    }
  }
}
```

## Cargo Build Behavior

- `build/<slot>/` becomes `CARGO_TARGET_DIR`
- when `CARGO_HOME` is not already set, `kbuild` uses `./.cargo-home/`
- `cargo build` builds the crate into the selected slot
- `cargo test --no-run` builds test binaries when `cargo.tests` is enabled
- `build/<slot>/sdk/` receives a crate snapshot based on `cargo.sdk.include`
- configured demo targets are staged under `demo/<demo>/build/<slot>/`

## Demo Mapping

Unlike the CMake flow, Cargo demos are defined explicitly in `cargo.demos`.
That lets the build config keep C++-style demo names such as `exe/core` while
mapping them to Cargo binaries or examples.
