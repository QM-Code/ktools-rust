# Ktrace Rust Documentation

`ktrace` is the Rust tracing and operator-logging layer for the ktools stack.

The Rust implementation is built around two public runtime types:

- `TraceLogger`
- `Logger`

`TraceLogger` is the library-facing source object. It owns one trace namespace,
declares channels in that namespace, and emits trace or log output.

`Logger` is the executable-facing runtime. It aggregates one or more
`TraceLogger` instances, enables channels by selector, formats output, and
builds the `kcli` inline parser for `--trace*` options.

## Start Here

- [API guide](api.md)
- [Selectors and CLI](selectors.md)
- [Examples](examples.md)

## Typical Flow

```rust
use ktrace::{Logger, TraceLogger, ktrace_trace, ktrace_info};

let logger = Logger::new();
let app_trace = TraceLogger::new("app")?;
app_trace.add_channel("startup", ktrace::color("BrightCyan")?)?;

logger.add_trace_logger(app_trace.clone())?;
logger.enable_channel("app.startup", "")?;

ktrace_trace!(app_trace, "startup", "starting {}", "demo")?;
ktrace_info!(app_trace, "operator-visible message")?;
# Ok::<(), ktrace::TraceError>(())
```

## Core Concepts

`Trace namespace`

- A named source domain such as `app`, `alpha`, or `gamma`.
- Created when a `TraceLogger` is constructed.

`Channel`

- A channel inside one namespace, such as `startup`, `net`, or
  `scheduler.tick`.
- Channels must be registered before they can be enabled and traced.

`Selector`

- A runtime enablement pattern such as `app.startup`, `.startup`, `*.*`, or
  `*.{net,io}`.
- Selectors are resolved against registered channels only.

`Operational log`

- `info`, `warn`, and `error` output is always emitted once a `TraceLogger` is
  attached to a `Logger`.
- It does not depend on channel enablement.

## Working References

- Public API: [`src/src/lib.rs`](../src/src/lib.rs)
- Demo binaries: [`src/src/bin/core.rs`](../src/src/bin/core.rs),
  [`src/src/bin/omega.rs`](../src/src/bin/omega.rs)
- Behavior coverage: [`src/tests/channel_semantics.rs`](../src/tests/channel_semantics.rs),
  [`src/tests/log_api.rs`](../src/tests/log_api.rs),
  [`src/tests/demo_cli.rs`](../src/tests/demo_cli.rs)
