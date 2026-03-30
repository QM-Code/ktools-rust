# kcli-rust

`ktools-rust/kcli/` is the Rust implementation of `kcli`.

It follows the same parsing model as the C++ SDK:

- top-level options such as `--verbose`
- inline roots such as `--alpha-*`, `--trace-*`, and `--build-*`
- aliases, required values, optional values, and positional dispatch
- full CLI validation before any handler runs

## Documentation

- [Overview and quick start](docs/index.md)
- [API guide](docs/api.md)
- [Parsing behavior](docs/behavior.md)
- [Examples](docs/examples.md)

## Quick Start

```rust
use kcli::{InlineParser, Parser};

let mut parser = Parser::new();
let mut build = InlineParser::new("--build")?;

build.set_value_handler(
    "-profile",
    |_context, _value| Ok(()),
    "Set build profile.",
)?;

parser.add_inline_parser(build)?;
parser.add_alias("-v", "--verbose", &[] as &[&str])?;
parser.set_flag_handler("--verbose", |_context| Ok(()), "Enable verbose logging.")?;

let argv = vec!["app", "--verbose"];
parser.parse(&argv)?;
# Ok::<(), Box<dyn std::error::Error>>(())
```

## Behavior Highlights

- `parse()` returns `kcli::CliError` for invalid CLI input and handler failures
- `parse_or_exit()` reports `[error] [cli] ...` and exits with code `2`
- bare inline roots such as `--build` print inline help unless a root value is provided
- required-value handlers may consume a first value token that begins with `-`
- literal `--` is rejected as an unknown option; it is not treated as an option terminator

## Build

From this component root:

```bash
kbuild --build-latest
```

If `kbuild` is not on `PATH`, use the shared workspace copy:

```bash
python3 ../../kbuild/kbuild.py --build-latest
```

Direct Cargo workflow:

```bash
cargo test --manifest-path Cargo.toml -p kcli
cargo test --manifest-path demo/exe/core/Cargo.toml
```

Direct Cargo commands in this component stage their `target` output under
`../build/cargo/kcli/`.

SDK staging after `kbuild --build-latest`:

- `build/latest/sdk/Cargo.toml`
- `build/latest/sdk/src/`
- `build/latest/sdk/tests/`

## Demos

- Bootstrap compile/link check: `demo/bootstrap/`
- SDK demos: `demo/sdk/{alpha,beta,gamma}`
- Executable demos: `demo/exe/{core,omega}`

After `kbuild --build-latest`, staged demo outputs appear under:

- `demo/bootstrap/build/latest/`
- `demo/sdk/alpha/build/latest/`
- `demo/sdk/beta/build/latest/`
- `demo/sdk/gamma/build/latest/`
- `demo/exe/core/build/latest/`
- `demo/exe/omega/build/latest/`

Useful demo commands:

```bash
./demo/bootstrap/build/latest/bootstrap
./demo/sdk/alpha/build/latest/sdk_alpha --alpha-message "hello"
./demo/sdk/beta/build/latest/sdk_beta --beta-workers 8
./demo/sdk/gamma/build/latest/sdk_gamma --gamma-tag "prod"
./demo/exe/core/build/latest/core --alpha
./demo/exe/core/build/latest/core --output stdout
./demo/exe/omega/build/latest/omega --build
./demo/exe/omega/build/latest/omega --newgamma-tag "prod"
```

## Component Layout

- Cargo workspace: `Cargo.toml`
- Public API: `src/lib.rs`
- Parser implementation: `src/{backend,model,normalize,process/}`
- Demo packages: `demo/{bootstrap,sdk/*,exe/*}/`
- Behavior coverage: `tests/`
