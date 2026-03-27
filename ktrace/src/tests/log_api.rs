mod common;

use common::capture_stdout;
use ktrace::{Logger, OutputOptions, TraceLogger};

#[test]
fn info_warn_and_error_messages_are_emitted() -> Result<(), Box<dyn std::error::Error>> {
    let logger = Logger::new();
    let trace = TraceLogger::new("tests")?;
    logger.add_trace_logger(trace.clone())?;
    logger.set_output_options(OutputOptions {
        filenames: true,
        line_numbers: true,
        function_names: false,
        timestamps: false,
    })?;

    let output = capture_stdout(|| {
        trace.info("info message").expect("info should log");
        trace.warn("warn value 7").expect("warn should log");
        trace.error("error message").expect("error should log");
    });

    assert!(output.starts_with("[tests] [info] "));
    assert!(output.contains("\n[tests] [warning] "));
    assert!(output.contains("\n[tests] [error] "));
    assert!(output.contains("info message"));
    assert!(output.contains("warn value 7"));
    assert!(output.contains("error message"));
    assert!(output.contains("log_api:"));
    Ok(())
}

#[test]
fn trace_output_respects_channel_enablement() -> Result<(), Box<dyn std::error::Error>> {
    let logger = Logger::new();
    let trace = TraceLogger::new("tests")?;
    trace.add_channel("trace", ktrace::color("BrightCyan")?)?;
    logger.add_trace_logger(trace.clone())?;
    logger.enable_channel("tests.trace", "")?;

    let output = capture_stdout(|| {
        trace.trace("trace", "member 42 {ok}").expect("trace should log");
    });

    assert!(output.contains("[tests] [trace]"));
    assert!(output.contains("member 42 {ok}"));
    Ok(())
}
