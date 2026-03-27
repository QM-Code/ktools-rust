use crate::{color, TraceLogger, TraceResult};

pub fn get_trace_logger() -> TraceResult<TraceLogger> {
    let trace = TraceLogger::new("gamma")?;
    trace.add_channel("scheduler", color("BrightMagenta")?)?;
    trace.add_channel("scheduler.tick", color("Orange3")?)?;
    Ok(trace)
}
