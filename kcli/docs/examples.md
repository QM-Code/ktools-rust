# Examples

This page shows a few common `kcli` patterns. For complete compiling examples,
also see:

- [`../src/src/bin/bootstrap.rs`](../src/src/bin/bootstrap.rs)
- [`../src/src/bin/sdk_alpha.rs`](../src/src/bin/sdk_alpha.rs)
- [`../src/src/bin/sdk_beta.rs`](../src/src/bin/sdk_beta.rs)
- [`../src/src/bin/sdk_gamma.rs`](../src/src/bin/sdk_gamma.rs)
- [`../src/src/bin/core.rs`](../src/src/bin/core.rs)
- [`../src/src/bin/omega.rs`](../src/src/bin/omega.rs)

## Minimal Executable

```rust
use kcli::Parser;

fn main() {
    let argv = std::env::args().collect::<Vec<_>>();
    let mut parser = Parser::new();

    parser
        .add_alias("-v", "--verbose", &[] as &[&str])
        .expect("alias should register");
    parser
        .set_flag_handler("--verbose", |_context| Ok(()), "Enable verbose logging.")
        .expect("handler should register");

    parser.parse_or_exit(&argv);
}
```

## Inline Root With Subcommands-Like Options

```rust
let mut parser = kcli::Parser::new();
let mut build = kcli::InlineParser::new("--build")?;

build.set_value_handler("-profile", |_context, _value| Ok(()), "Set build profile.")?;
build.set_flag_handler("-clean", |_context| Ok(()), "Enable clean build.")?;

parser.add_inline_parser(build)?;
```

This enables:

```text
--build
--build-profile release
--build-clean
```

## Bare Root Value Handler

```rust
let mut config = kcli::InlineParser::new("--config")?;

config.set_root_value_handler_with_help(
    |_context, _value| Ok(()),
    "<assignment>",
    "Store a config assignment.",
)?;
```

This enables:

```text
--config
--config user=alice
```

Behavior:

- `--config` prints inline help
- `--config user=alice` invokes the root value handler

## Alias Preset Tokens

```rust
let mut parser = kcli::Parser::new();

parser.add_alias("-c", "--config-load", &["user-file"])?;
parser.set_value_handler("--config-load", |_context, _value| Ok(()), "Load config.")?;
```

This makes:

```text
-c settings.json
```

behave like:

```text
--config-load user-file settings.json
```

Inside the handler:

- `context.option` is `--config-load`
- `context.value_tokens` is `["user-file", "settings.json"]`

## Optional Values

```rust
parser.set_optional_value_handler(
    "--color",
    |_context, _value| Ok(()),
    "Set or auto-detect color output.",
)?;
```

This enables both:

```text
--color
--color always
```

## Positionals

```rust
parser.set_positional_handler(|context| {
    for token in &context.value_tokens {
        println!("{token}");
    }
    Ok(())
})?;
```

The positional handler receives all remaining non-option tokens after option
parsing succeeds.
