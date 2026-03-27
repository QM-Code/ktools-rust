use crate::{color, TraceLogger, TraceResult};

pub fn get_trace_logger() -> TraceResult<TraceLogger> {
    let trace = TraceLogger::new("alpha")?;
    trace.add_channel("net", color("DeepSkyBlue1")?)?;
    trace.add_channel("cache", color("Gold3")?)?;
    Ok(trace)
}
