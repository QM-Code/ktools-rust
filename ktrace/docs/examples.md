# Examples

## Minimal Executable

```rust
use kcli::Parser;
use ktrace::{Logger, TraceLogger, ktrace_info, ktrace_trace};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let logger = Logger::new();
    let app_trace = TraceLogger::new("app")?;
    app_trace.add_channel("startup", ktrace::color("BrightCyan")?)?;

    logger.add_trace_logger(app_trace.clone())?;

    let mut parser = Parser::new();
    parser.add_inline_parser(logger.make_inline_parser(app_trace.clone(), "trace")?)?;

    let argv = std::env::args().collect::<Vec<_>>();
    parser.parse_or_exit(&argv);

    logger.enable_channel("app.startup", "")?;
    ktrace_trace!(app_trace, "startup", "starting {}", "demo")?;
    ktrace_info!(app_trace, "operator message")?;
    Ok(())
}
```

## Library-Style Shared Trace Source

```rust
use ktrace::TraceLogger;

fn get_trace_logger() -> Result<TraceLogger, ktrace::TraceError> {
    let trace = TraceLogger::new("alpha")?;
    trace.add_channel("net", ktrace::color("DeepSkyBlue1")?)?;
    trace.add_channel("net.alpha", ktrace::DEFAULT_COLOR)?;
    trace.add_channel("cache", ktrace::color("Gold3")?)?;
    Ok(trace)
}
```

## Output Formatting

```rust
logger.set_output_options(ktrace::OutputOptions {
    filenames: true,
    line_numbers: true,
    function_names: true,
    timestamps: true,
})?;
```

Example prefix shapes:

- `[app] [startup]`
- `[app] [info]`
- `[app] [1711500000.123456] [startup] [main:42]`
- `[app] [warning] [main:42:run]`

## Trace-Changed Usage

```rust
use ktrace::ktrace_trace_changed;

let trace = TraceLogger::new("app")?;
trace.add_channel("state", ktrace::DEFAULT_COLOR)?;

ktrace_trace_changed!(trace, "state", "connected", "state changed")?;
```

The second call with the same key at the same call site is suppressed until the
key changes.

## Where To See Running Examples

- [`src/src/bin/bootstrap.rs`](../src/src/bin/bootstrap.rs)
- [`src/src/bin/sdk_alpha.rs`](../src/src/bin/sdk_alpha.rs)
- [`src/src/bin/sdk_beta.rs`](../src/src/bin/sdk_beta.rs)
- [`src/src/bin/sdk_gamma.rs`](../src/src/bin/sdk_gamma.rs)
- [`src/src/bin/core.rs`](../src/src/bin/core.rs)
- [`src/src/bin/omega.rs`](../src/src/bin/omega.rs)
- [`src/tests/demo_cli.rs`](../src/tests/demo_cli.rs)
