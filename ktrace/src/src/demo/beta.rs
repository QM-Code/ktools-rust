use crate::{color, TraceLogger, TraceResult};

pub fn get_trace_logger() -> TraceResult<TraceLogger> {
    let trace = TraceLogger::new("beta")?;
    trace.add_channel("io", color("BrightGreen")?)?;
    trace.add_channel("workers", color("BrightYellow")?)?;
    Ok(trace)
}
