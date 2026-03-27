# API Guide

This page summarizes the public types in
[`../src/src/lib.rs`](../src/src/lib.rs).

## Core Types

| Type | Purpose |
| --- | --- |
| `kcli::Parser` | Owns aliases, top-level handlers, positional handling, and inline parser registration. |
| `kcli::InlineParser` | Defines one inline root namespace such as `--build` plus its `--build-*` handlers. |
| `kcli::HandlerContext` | Metadata delivered to flag, value, and positional handlers. |
| `kcli::CliError` | Error returned by `parse()` for invalid CLI input and handler failures. |
| `kcli::ConfigError` | Error returned while registering invalid roots, options, aliases, or help text. |

## HandlerContext

`HandlerContext` is passed to every handler.

| Field | Meaning |
| --- | --- |
| `root` | Inline root name without leading dashes, such as `build`. Empty for top-level handlers and positional dispatch. |
| `option` | Effective option token after alias expansion, such as `--verbose` or `--build-profile`. Empty for positional dispatch. |
| `command` | Normalized command name without leading dashes. Empty for positional dispatch and inline root value handlers. |
| `value_tokens` | Effective value tokens after alias expansion. Tokens from the shell are preserved verbatim; alias preset tokens are prepended. |

## InlineParser

### Construction

```rust
let parser = kcli::InlineParser::new("--build")?;
```

The root may be provided as either:

- `"build"`
- `"--build"`

### Root Value Handler

```rust
parser.set_root_value_handler(|_context, _value| Ok(()))?;
parser.set_root_value_handler_with_help(
    |_context, _value| Ok(()),
    "<selector>",
    "Select build targets.",
)?;
```

The root value handler processes the bare root form, for example:

- `--build release`
- `--config user.json`

If the bare root is used without a value, `kcli` prints inline help for that
root instead.

### Inline Handlers

```rust
parser.set_flag_handler("-clean", |_context| Ok(()), "Enable clean build.")?;
parser.set_value_handler("-profile", |_context, _value| Ok(()), "Set build profile.")?;
parser.set_optional_value_handler("-enable", |_context, _value| Ok(()), "Enable build mode.")?;
```

Inline handler options may be written in either form:

- short inline form: `-profile`
- fully-qualified form: `--build-profile`

## Parser

### Top-Level Handlers

```rust
parser.set_flag_handler("--verbose", |_context| Ok(()), "Enable verbose logging.")?;
parser.set_value_handler("--output", |_context, _value| Ok(()), "Set output target.")?;
parser.set_optional_value_handler("--color", |_context, _value| Ok(()), "Set or auto-detect color output.")?;
```

Top-level handler options may be written as either:

- `"verbose"`
- `"--verbose"`

### Aliases

```rust
parser.add_alias("-v", "--verbose", &[] as &[&str])?;
parser.add_alias("-c", "--config-load", &["user-file"])?;
```

Rules:

- aliases use single-dash form such as `-v`
- alias targets use double-dash form such as `--verbose`
- preset tokens are prepended to the handler's effective `value_tokens`

### Positional Handler

```rust
parser.set_positional_handler(|context| {
    for token in &context.value_tokens {
        println!("{token}");
    }
    Ok(())
})?;
```

The positional handler receives remaining non-option tokens in
`HandlerContext::value_tokens`.

### Inline Parser Registration

```rust
parser.add_inline_parser(build_parser)?;
```

Duplicate inline roots are rejected.

### Parse Entry Points

```rust
parser.parse(&argv)?;
parser.parse_or_exit(&argv);
```

`parse()`

- preserves the caller's input token list
- returns `CliError`
- does not run handlers until the full command line validates

`parse_or_exit()`

- preserves the caller's input token list
- reports invalid CLI input to `stderr` as `[error] [cli] ...`
- exits with code `2`

## Value Handler Registration

Use the registration form that matches the CLI contract you want:

- `set_flag_handler(option, handler, description)` for flag-style options
- `set_value_handler(option, handler, description)` for required values
- `set_optional_value_handler(option, handler, description)` for optional values
- `set_root_value_handler*` for bare inline roots such as `--build release`
