#![allow(dead_code)]

use std::sync::OnceLock;

use ktrace::{color, TraceError, TraceLogger};

pub fn get_trace_logger() -> Result<TraceLogger, TraceError> {
    static TRACE: OnceLock<TraceLogger> = OnceLock::new();

    if let Some(trace) = TRACE.get() {
        return Ok(trace.clone());
    }

    let trace = TraceLogger::new("beta")?;
    trace.add_channel("io", color("MediumSpringGreen")?)?;
    trace.add_channel("scheduler", color("Orange3")?)?;
    let _ = TRACE.set(trace.clone());
    Ok(TRACE.get().cloned().unwrap_or(trace))
}

pub fn test_trace_logging_channels() -> Result<(), TraceError> {
    let trace = get_trace_logger()?;
    trace.trace("io", "beta trace test on channel 'io'")?;
    trace.trace("scheduler", "beta trace test on channel 'scheduler'")?;
    Ok(())
}
