# Selectors And CLI

`ktrace` uses selectors to enable or disable registered channels at runtime.

## Exact Selectors

Use exact selectors with `Logger::enable_channel()`,
`Logger::disable_channel()`, and `Logger::should_trace_channel()`.

Forms:

- `namespace.channel`
- `.channel`

Examples:

- `app.startup`
- `alpha.net`
- `.startup` with local namespace `app`

Exact selectors must resolve to a valid registered channel path.

## Selector Lists

Use selector lists with `Logger::enable_channels()` and
`Logger::disable_channels()`.

Selector lists support:

- CSV: `alpha.net,beta.io`
- wildcard namespace: `*.net`
- wildcard channel segments: `alpha.*`, `*.*`, `*.*.*`
- brace expansion: `*.{net,io}`, `{alpha,beta}.scheduler`
- local-namespace shorthand: `.startup,.net`

Examples:

```text
app.*
app.*.*
*.net
*.{net,io}
{alpha,beta}.scheduler.tick
```

## Resolution Rules

- selectors are resolved against registered channels only
- unmatched selectors do not create new channels
- empty selector lists are rejected
- invalid selector syntax raises `TraceError`
- `*.*` includes top-level channels across namespaces
- `*.*.*` includes channels up to depth 3

## `kcli` Inline Parser

`Logger::make_inline_parser()` exposes the selector system through `kcli`.

Typical setup:

```rust
let logger = ktrace::Logger::new();
let app_trace = ktrace::TraceLogger::new("app")?;
app_trace.add_channel("startup", ktrace::color("BrightCyan")?)?;
logger.add_trace_logger(app_trace.clone())?;

let mut parser = kcli::Parser::new();
parser.add_inline_parser(logger.make_inline_parser(app_trace.clone(), "trace")?)?;
# Ok::<(), Box<dyn std::error::Error>>(())
```

Common commands:

```text
--trace 'app.startup'
--trace '*.net'
--trace '*.{net,io}'
--trace-namespaces
--trace-channels
--trace-colors
--trace-files
--trace-functions
--trace-timestamps
```
