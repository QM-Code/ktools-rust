mod common;

use common::capture_stdout;
use ktrace::{ktrace_trace_changed, Logger, TraceLogger};

fn make_trace() -> Result<(Logger, TraceLogger), Box<dyn std::error::Error>> {
    let logger = Logger::new();
    let trace = TraceLogger::new("tests")?;
    trace.add_channel("changed", ktrace::color("BrightYellow")?)?;
    logger.add_trace_logger(trace.clone())?;
    logger.enable_channel("tests.changed", "")?;
    Ok((logger, trace))
}

fn emit_changed(trace: &TraceLogger, key: &str) {
    ktrace_trace_changed!(trace, "changed", key, "changed {}", key)
        .expect("trace_changed should log");
}

#[test]
fn trace_changed_suppresses_duplicates_at_the_same_call_site(
) -> Result<(), Box<dyn std::error::Error>> {
    let (_logger, trace) = make_trace()?;

    let output = capture_stdout(|| {
        emit_changed(&trace, "alpha");
        emit_changed(&trace, "alpha");
        emit_changed(&trace, "beta");
        emit_changed(&trace, "beta");
    });

    assert_eq!(output.matches("changed alpha").count(), 1);
    assert_eq!(output.matches("changed beta").count(), 1);
    Ok(())
}

#[test]
fn trace_changed_uses_call_site_as_part_of_the_deduplication_key(
) -> Result<(), Box<dyn std::error::Error>> {
    let (_logger, trace) = make_trace()?;

    let output = capture_stdout(|| {
        ktrace_trace_changed!(trace, "changed", "same", "first call site")
            .expect("first call site should log");
        ktrace_trace_changed!(trace, "changed", "same", "second call site")
            .expect("second call site should log");
    });

    assert_eq!(output.matches("first call site").count(), 1);
    assert_eq!(output.matches("second call site").count(), 1);
    Ok(())
}
