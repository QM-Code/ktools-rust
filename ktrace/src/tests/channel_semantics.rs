mod common;

use ktrace::{color, Logger, TraceLogger};


fn add_test_channels(logger: &Logger) -> Result<(), Box<dyn std::error::Error>> {
    let trace = TraceLogger::new("tests")?;
    trace.add_channel("net", color("DeepSkyBlue1")?)?;
    trace.add_channel("cache", color("Gold3")?)?;
    trace.add_channel("store", color("BrightBlue")?)?;
    trace.add_channel("store.requests", color("BrightBlue")?)?;
    logger.add_trace_logger(trace)?;
    Ok(())
}

#[test]
fn explicit_enable_disable_semantics_work() -> Result<(), Box<dyn std::error::Error>> {
    let logger = Logger::new();
    add_test_channels(&logger)?;

    logger.enable_channels("tests.*", "tests")?;
    assert!(logger.should_trace_channel("tests.net", ""));
    assert!(logger.should_trace_channel("tests.cache", ""));

    logger.disable_channels("tests.*", "tests")?;
    assert!(!logger.should_trace_channel("tests.net", ""));
    assert!(!logger.should_trace_channel("tests.cache", ""));

    logger.enable_channel("tests.net", "")?;
    assert!(logger.should_trace_channel("tests.net", ""));
    assert!(!logger.should_trace_channel("tests.cache", ""));

    logger.disable_channel("tests.net", "")?;
    assert!(!logger.should_trace_channel("tests.net", ""));
    Ok(())
}

#[test]
fn selector_lists_only_enable_registered_channels() -> Result<(), Box<dyn std::error::Error>> {
    let logger = Logger::new();
    add_test_channels(&logger)?;

    logger.enable_channels("*.*.*", "tests")?;
    assert!(logger.should_trace_channel("tests.store.requests", ""));
    assert!(logger.should_trace_channel("tests.net", ""));
    assert!(!logger.should_trace_channel("tests.missing.child", ""));

    logger.disable_channels("*.{net,cache}", "tests")?;
    assert!(!logger.should_trace_channel("tests.net", ""));
    assert!(!logger.should_trace_channel("tests.cache", ""));
    assert!(logger.should_trace_channel("tests.store.requests", ""));
    Ok(())
}

#[test]
fn duplicate_namespaces_merge_but_conflicting_colors_fail(
) -> Result<(), Box<dyn std::error::Error>> {
    let logger = Logger::new();

    let first = TraceLogger::new("tests")?;
    first.add_channel("net", color("Gold3")?)?;
    logger.add_trace_logger(first)?;

    let second = TraceLogger::new("tests")?;
    second.add_channel("net", color("Gold3")?)?;
    logger.add_trace_logger(second)?;

    let third = TraceLogger::new("tests")?;
    third.add_channel("net", color("Orange3")?)?;
    let error = logger.add_trace_logger(third).unwrap_err();
    assert!(error
        .to_string()
        .contains("conflicting trace color for 'tests.net'"));
    Ok(())
}
