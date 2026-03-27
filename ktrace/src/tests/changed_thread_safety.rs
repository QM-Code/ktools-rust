use std::sync::Arc;

use ktrace::{ktrace_trace_changed, Logger, TraceLogger};

fn get_trace_logger() -> Result<TraceLogger, Box<dyn std::error::Error>> {
    let trace = TraceLogger::new("tests")?;
    trace.add_channel("changed", ktrace::color("BrightYellow")?)?;
    Ok(trace)
}

#[test]
fn trace_changed_is_thread_safe() -> Result<(), Box<dyn std::error::Error>> {
    let logger = Logger::new();
    let trace = Arc::new(get_trace_logger()?);
    logger.add_trace_logger((*trace).clone())?;

    let mut workers = Vec::new();
    for thread_index in 0..8 {
        let trace = Arc::clone(&trace);
        workers.push(std::thread::spawn(move || {
            for iteration in 0..10_000 {
                let key = format!("{}:{}", thread_index, iteration & 1);
                ktrace_trace_changed!(trace, "changed", key, "changed")
                    .expect("trace_changed should not fail");
            }
        }));
    }

    for worker in workers {
        worker.join().expect("thread should finish");
    }

    Ok(())
}
