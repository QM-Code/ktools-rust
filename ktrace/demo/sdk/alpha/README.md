# Alpha Demo SDK

Exists for CI and as a minimal reference for integrating an SDK add-on with the
Rust `ktrace` crate.

This SDK demonstrates the library-side pattern:

- expose `get_trace_logger()`
- build a shared `TraceLogger("alpha")` with local channels
- emit with `trace.trace(...)` and `trace.info()/warn()/error()`
