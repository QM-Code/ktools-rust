use std::error::Error;

use ktrace::demo::beta::{get_trace_logger, test_trace_logging_channels};
use ktrace::Logger;

fn main() -> Result<(), Box<dyn Error>> {
    let logger = Logger::new();
    let trace = get_trace_logger()?;

    logger.add_trace_logger(trace)?;
    logger.enable_channels("beta.*.*.*", "beta")?;
    test_trace_logging_channels()?;

    println!("KTRACE rust beta demo SDK check passed");
    Ok(())
}
