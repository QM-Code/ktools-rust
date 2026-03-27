# ktrace-rust

`ktools-rust/ktrace/` is the Rust implementation of `ktrace`.

It provides:

- namespaced trace channels through `TraceLogger`
- operator-facing `info`, `warn`, and `error` logging
- selector-based runtime enablement through `Logger`
- `kcli` inline parser integration for `--trace*` controls

## Documentation

- [Overview](docs/index.md)
- [API guide](docs/api.md)
- [Selectors and CLI](docs/selectors.md)
- [Examples](docs/examples.md)

## Build

From this repo root:

```bash
./kbuild.py --build-latest
```

Direct Cargo workflow:

```bash
cd src
cargo test
```

## Demos

The repo exposes demo binaries through the local Rust `kbuild` config:

- `exe/core`
- `exe/omega`

After `./kbuild.py --build-latest`, staged demo outputs appear under:

- `demo/exe/core/build/latest/`
- `demo/exe/omega/build/latest/`
