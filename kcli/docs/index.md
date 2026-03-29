# Kcli Rust Documentation

`kcli` is the Rust command-line parsing layer for the ktools stack.

It is intentionally opinionated about normal CLI behavior:

- parse first
- fail early on invalid input
- do not run handlers until the full command line validates
- preserve the caller's input token list
- support grouped inline roots such as `--trace-*` and `--build-*`

## Start Here

- [API guide](api.md)
- [Parsing behavior](behavior.md)
- [Examples](examples.md)

## Typical Flow

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

## Core Concepts

`Parser`

- Owns top-level handlers, aliases, positional handling, and inline parser
  registration.

`InlineParser`

- Defines one inline root namespace such as `--alpha`, `--trace`, or `--build`.

`HandlerContext`

- Exposes the effective option, command, root, and value tokens seen by the
  handler after alias expansion.

`CliError`

- Returned from `Parser::parse()` when the CLI is invalid or a handler returns
  an error string.

## Which Entry Point Should I Use?

Use `parse_or_exit()` when:

- you are in a normal executable `main()`
- invalid CLI input should print a standardized error and exit with code `2`
- you do not need custom formatting or recovery

Use `parse()` when:

- you want to test parse failures directly
- you want custom error formatting or exit codes
- you need to intercept handler failures

## Build And Explore

```bash
kbuild --build-latest
./demo/sdk/alpha/build/latest/sdk_alpha --alpha-message "hello"
./demo/exe/core/build/latest/core --alpha
./demo/exe/omega/build/latest/omega --build
```

## Working References

If you want complete, compiling examples, start with:

- [`../demo/bootstrap/src/main.rs`](../demo/bootstrap/src/main.rs)
- [`../demo/sdk/alpha/src/lib.rs`](../demo/sdk/alpha/src/lib.rs)
- [`../demo/sdk/alpha/src/main.rs`](../demo/sdk/alpha/src/main.rs)
- [`../demo/sdk/beta/src/lib.rs`](../demo/sdk/beta/src/lib.rs)
- [`../demo/sdk/gamma/src/lib.rs`](../demo/sdk/gamma/src/lib.rs)
- [`../demo/exe/core/src/main.rs`](../demo/exe/core/src/main.rs)
- [`../demo/exe/omega/src/main.rs`](../demo/exe/omega/src/main.rs)
- [`../tests/api.rs`](../tests/api.rs)

The public API contract lives in [`../src/lib.rs`](../src/lib.rs).
