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

## Quick Start

```rust
use ktrace::{Logger, TraceLogger, ktrace_info, ktrace_trace};

let logger = Logger::new();
let app_trace = TraceLogger::new("app")?;
app_trace.add_channel("startup", ktrace::color("BrightCyan")?)?;

logger.add_trace_logger(app_trace.clone())?;
logger.enable_channel("app.startup", "")?;

ktrace_trace!(app_trace, "startup", "starting {}", "demo")?;
ktrace_info!(app_trace, "operator-visible message")?;
# Ok::<(), ktrace::TraceError>(())
```

## API Model

`TraceLogger` is the library-facing source object. Construct it with an
explicit namespace and declare channels on it:

```rust
let trace = ktrace::TraceLogger::new("alpha")?;
trace.add_channel("net", ktrace::color("DeepSkyBlue1")?)?;
trace.add_channel("cache", ktrace::color("Gold3")?)?;
# Ok::<(), ktrace::TraceError>(())
```

SDKs typically expose a shared handle from `get_trace_logger()` so executable
code can import registered namespaces and channels without rebuilding them.

`Logger` is the executable-facing runtime. It imports one or more
`TraceLogger`s, maintains the channel registry, owns selector enablement, and
builds the `kcli` inline parser for `--trace*` options.

## Logging APIs

Channel-based trace output:

```rust
trace.trace("channel", "plain trace message")?;
trace.trace_changed("channel", "session:42", "only when key changes")?;
# Ok::<(), ktrace::TraceError>(())
```

Always-visible operational logging:

```rust
trace.info("service started")?;
trace.warn("retrying connection")?;
trace.error("startup failed")?;
# Ok::<(), ktrace::TraceError>(())
```

Operational logging is independent of channel enablement. Macro helpers use
standard Rust `format!` semantics and capture file, line, and module
information automatically.

## CLI Integration

Pass the executable's local `TraceLogger` to `make_inline_parser()` so
leading-dot selectors resolve against the intended namespace:

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
--trace '*.*.*.*'
--trace-namespaces
--trace-channels
--trace-colors
--trace-files
--trace-functions
--trace-timestamps
```

## Selector Forms

Single-selector APIs on `Logger`:

- `.channel[.sub[.sub]]` for a local channel in the provided local namespace
- `namespace.channel[.sub[.sub]]` for an explicit namespace

List APIs on `Logger`:

- `enable_channels(...)`
- `disable_channels(...)`
- selector lists support CSV, `*`, brace expansion, and leading-dot local selectors
- list selectors resolve against the channels currently registered at call time
- empty selector lists are rejected
- unregistered channels remain disabled even when a selector pattern would otherwise match

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
cd src
cargo test
```

Direct Cargo commands in this component stage their `target` output under
`../build/cargo/ktrace/`.

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
./demo/sdk/alpha/build/latest/sdk_alpha
./demo/sdk/beta/build/latest/sdk_beta
./demo/sdk/gamma/build/latest/sdk_gamma
./demo/exe/core/build/latest/core --trace
./demo/exe/omega/build/latest/omega --trace '*.*'
./demo/exe/omega/build/latest/omega --trace-namespaces
./demo/exe/omega/build/latest/omega --trace-colors
```

## Component Layout

- Public API: `src/src/lib.rs`
- Color catalog: `src/src/colors.rs`
- Demo binaries: `src/src/bin/`
- Demo-owned support modules: `src/src/bin/support/sdk_{alpha,beta,gamma}.rs`
- Behavior coverage: `src/tests/`
