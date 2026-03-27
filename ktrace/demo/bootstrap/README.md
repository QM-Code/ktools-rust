# Bootstrap Demo

Exists for CI and as the smallest compile/link usage reference for the Rust
`ktrace` crate.

This demo shows the minimal executable-side setup:

- create a `ktrace::Logger`
- create a local `ktrace::TraceLogger("bootstrap")`
- add one or more channels
- `logger.add_trace_logger(...)`
- enable local selectors through `logger.enable_channel(".channel", trace.namespace())`
- emit with `ktrace_trace!(...)`
