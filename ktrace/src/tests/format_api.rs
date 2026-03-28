mod common;

use common::capture_stdout;
use ktrace::{ktrace_trace, ktrace_warn, Logger, OutputOptions, TraceLogger};

#[test]
fn warn_macro_uses_rust_formatting_and_source_location() -> Result<(), Box<dyn std::error::Error>> {
    let logger = Logger::new();
    let trace = TraceLogger::new("tests")?;
    logger.add_trace_logger(trace.clone())?;
    logger.set_output_options(OutputOptions {
        filenames: true,
        line_numbers: true,
        function_names: false,
        timestamps: false,
    })?;

    let warn_line = line!() + 2;
    let output =
        capture_stdout(|| ktrace_warn!(trace, "escaped {{}} {}", 7).expect("warn should log"));

    assert!(output.contains("escaped {} 7"));
    assert!(output.contains(&format!("format_api:{warn_line}")));
    Ok(())
}

#[test]
fn trace_macro_formats_enabled_channel_output() -> Result<(), Box<dyn std::error::Error>> {
    let logger = Logger::new();
    let trace = TraceLogger::new("tests")?;
    trace.add_channel("trace", ktrace::color("BrightCyan")?)?;
    logger.add_trace_logger(trace.clone())?;
    logger.enable_channel("tests.trace", "")?;

    let output = capture_stdout(|| {
        ktrace_trace!(trace, "trace", "member {} {{ok}}", 42).expect("trace should log")
    });

    assert!(output.contains("[tests] [trace]"));
    assert!(output.contains("member 42 {ok}"));
    Ok(())
}
