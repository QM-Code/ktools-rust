use std::sync::OnceLock;

use crate::{color, TraceLogger, TraceResult, DEFAULT_COLOR};

pub fn get_trace_logger() -> TraceResult<TraceLogger> {
    static TRACE: OnceLock<TraceLogger> = OnceLock::new();

    if let Some(trace) = TRACE.get() {
        return Ok(trace.clone());
    }

    let trace = TraceLogger::new("alpha")?;
    trace.add_channel("net", color("DeepSkyBlue1")?)?;
    trace.add_channel("net.alpha", DEFAULT_COLOR)?;
    trace.add_channel("net.beta", DEFAULT_COLOR)?;
    trace.add_channel("net.gamma", DEFAULT_COLOR)?;
    trace.add_channel("net.gamma.deep", DEFAULT_COLOR)?;
    trace.add_channel("cache", color("Gold3")?)?;
    trace.add_channel("cache.gamma", color("Gold3")?)?;
    trace.add_channel("cache.delta", DEFAULT_COLOR)?;
    trace.add_channel("cache.special", color("Red")?)?;
    let _ = TRACE.set(trace.clone());
    Ok(TRACE.get().cloned().unwrap_or(trace))
}

pub fn test_trace_logging_channels() -> TraceResult<()> {
    let trace = get_trace_logger()?;
    trace.trace("net", "testing...")?;
    trace.trace("net.alpha", "testing...")?;
    trace.trace("net.beta", "testing...")?;
    trace.trace("net.gamma", "testing...")?;
    trace.trace("net.gamma.deep", "testing...")?;
    trace.trace("cache", "testing...")?;
    trace.trace("cache.gamma", "testing...")?;
    trace.trace("cache.delta", "testing...")?;
    trace.trace("cache.special", "testing...")?;
    Ok(())
}

pub fn test_standard_logging_channels() -> TraceResult<()> {
    let trace = get_trace_logger()?;
    trace.info("testing...")?;
    trace.warn("testing...")?;
    trace.error("testing...")?;
    Ok(())
}
