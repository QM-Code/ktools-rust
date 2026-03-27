# API Guide

This page summarizes the public API exported from
[`src/src/lib.rs`](../src/src/lib.rs).

## Core Types

| Type | Purpose |
| --- | --- |
| `TraceLogger` | Library-facing trace source for one namespace. |
| `Logger` | Executable-facing registry, selector runtime, formatter, and CLI integration point. |
| `OutputOptions` | Output formatting toggles for file, line, function, and timestamp labels. |
| `SourceLocation` | Explicit source metadata for low-level trace/log calls. |
| `TraceError` | Error type used by fallible `ktrace` operations. |
| `Severity` | Log severity enum used by low-level logging calls. |

## Shared Types

### `ColorId` and `DEFAULT_COLOR`

- `ColorId` is an alias for `u16`.
- `DEFAULT_COLOR` means “no explicit channel color”.
- Use `color("Name")` to resolve named colors instead of hard-coding ids.

### `OutputOptions`

| Field | Meaning |
| --- | --- |
| `filenames` | Include a source label such as `[lib:42]`. |
| `line_numbers` | Include the source line when `filenames` is enabled. |
| `function_names` | Include the function/module label when `filenames` is enabled. |
| `timestamps` | Include a compact epoch-based timestamp label. |

## `TraceLogger`

### Construction

```rust
let trace = ktrace::TraceLogger::new("alpha")?;
```

Rules:

- the namespace must be non-empty
- the namespace may contain ASCII letters, digits, `_`, and `-`

### Channel Registration

```rust
trace.add_channel("net", ktrace::color("DeepSkyBlue1")?)?;
trace.add_channel("cache", ktrace::DEFAULT_COLOR)?;
trace.add_channel("scheduler.tick", ktrace::color("Orange3")?)?;
```

Rules:

- channel depth is limited to 3 segments
- nested channels require their parent to be registered first
- conflicting explicit colors for the same qualified channel are rejected

### Querying

```rust
let namespace = trace.namespace();
let enabled = trace.should_trace_channel("net");
```

`should_trace_channel()` returns `false` when:

- the channel name is invalid
- the `TraceLogger` is not attached to a `Logger`
- the channel is registered but not enabled

### Emitting Trace Output

```rust
trace.trace("net", "plain trace message")?;
trace.trace_changed("net", "session:42", "only when key changes")?;
```

Low-level forms:

- `trace_with_location(channel, location, message)`
- `trace_changed_with_location(channel, key, location, message)`

`trace_changed*` suppresses duplicate messages for the same call site, channel,
and key.

### Emitting Operational Logs

```rust
trace.info("service started")?;
trace.warn("retrying connection")?;
trace.error("startup failed")?;
```

Low-level form:

- `log_with_location(severity, location, message)`

Operational logging is independent of channel enablement.

## `Logger`

### Construction

```rust
let logger = ktrace::Logger::new();
```

`Logger::default()` is equivalent.

### Attaching Trace Sources

```rust
logger.add_trace_logger(app_trace.clone())?;
logger.add_trace_logger(alpha_trace.clone())?;
```

Rules:

- one `TraceLogger` may only be attached to one `Logger`
- duplicate namespaces are merged
- registered channels become visible for selector resolution and help output

### Channel Enablement

```rust
logger.enable_channel("app.startup", "")?;
logger.enable_channel(".startup", "app")?;
logger.enable_channels("app.*,{alpha,beta}.net", "app")?;

logger.disable_channel("app.startup", "")?;
logger.disable_channels("*.{net,io}", "app")?;

let enabled = logger.should_trace_channel("app.startup", "");
```

Semantics:

- exact selectors require `namespace.channel` or `.channel`
- bulk selector APIs resolve only against currently registered channels
- unmatched selector patterns do not enable phantom channels
- invalid exact selectors raise `TraceError`
- invalid list selectors raise `TraceError`

### Output Control

```rust
logger.set_output_options(ktrace::OutputOptions {
    filenames: true,
    line_numbers: true,
    function_names: false,
    timestamps: true,
})?;

let options = logger.get_output_options()?;
```

### Registry Introspection

```rust
let namespaces = logger.get_namespaces()?;
let channels = logger.get_channels("alpha")?;
```

## `kcli` Integration

```rust
let parser = logger.make_inline_parser(app_trace.clone(), "trace")?;
```

This returns a `kcli::InlineParser` that supports:

- `--trace <selectors>`
- `--trace-examples`
- `--trace-namespaces`
- `--trace-channels`
- `--trace-colors`
- `--trace-files`
- `--trace-functions`
- `--trace-timestamps`

## Public Helper Functions

### `color(name)`

Resolves a named color such as:

- `BrightCyan`
- `DeepSkyBlue1`
- `Gold3`
- `Orange3`

### `available_color_names()`

Returns the set of color names supported by `color(name)`.

## Macros

These macros capture `file!()`, `line!()`, and `module_path!()` automatically.

| Macro | Purpose |
| --- | --- |
| `ktrace_trace!(logger, channel, ...)` | Trace a formatted message through one channel. |
| `ktrace_trace_changed!(logger, channel, key, ...)` | Trace only when the key changes at that call site. |
| `ktrace_info!(logger, ...)` | Emit an `info` operational log. |
| `ktrace_warn!(logger, ...)` | Emit a `warning` operational log. |
| `ktrace_error!(logger, ...)` | Emit an `error` operational log. |
