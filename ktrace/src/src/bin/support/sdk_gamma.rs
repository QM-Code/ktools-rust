#![allow(dead_code)]

use std::sync::OnceLock;

use ktrace::{color, TraceError, TraceLogger};

pub fn get_trace_logger() -> Result<TraceLogger, TraceError> {
    static TRACE: OnceLock<TraceLogger> = OnceLock::new();

    if let Some(trace) = TRACE.get() {
        return Ok(trace.clone());
    }

    let trace = TraceLogger::new("gamma")?;
    trace.add_channel("physics", color("MediumOrchid1")?)?;
    trace.add_channel("metrics", color("LightSkyBlue1")?)?;
    let _ = TRACE.set(trace.clone());
    Ok(TRACE.get().cloned().unwrap_or(trace))
}

pub fn test_trace_logging_channels() -> Result<(), TraceError> {
    let trace = get_trace_logger()?;
    trace.trace("physics", "gamma trace test on channel 'physics'")?;
    trace.trace("metrics", "gamma trace test on channel 'metrics'")?;
    Ok(())
}
