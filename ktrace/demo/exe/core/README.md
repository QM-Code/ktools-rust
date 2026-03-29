# Core Demo

Basic local-plus-imported tracing showcase for the Rust `ktrace` crate and the
alpha demo SDK.

This demo shows:

- executable-local tracing defined with a local `TraceLogger`
- imported SDK tracing added via the alpha demo support module's `get_trace_logger()`
- logger-managed selector state and output formatting
- local CLI integration through `parser.add_inline_parser(logger.make_inline_parser(...))`
