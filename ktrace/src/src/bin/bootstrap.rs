use std::error::Error;

use ktrace::{ktrace_trace, Logger, TraceLogger};

fn main() -> Result<(), Box<dyn Error>> {
    let logger = Logger::new();
    let trace = TraceLogger::new("bootstrap")?;
    trace.add_channel("bootstrap", ktrace::color("BrightGreen")?)?;

    logger.add_trace_logger(trace.clone())?;
    logger.enable_channel(".bootstrap", trace.namespace())?;
    ktrace_trace!(trace, "bootstrap", "ktrace bootstrap compile/link check")?;

    println!("Bootstrap succeeded.");
    Ok(())
}
