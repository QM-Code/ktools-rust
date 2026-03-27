use std::error::Error;

use ktrace::demo::alpha::{
    get_trace_logger, test_standard_logging_channels, test_trace_logging_channels,
};
use ktrace::Logger;

fn main() -> Result<(), Box<dyn Error>> {
    let logger = Logger::new();
    let trace = get_trace_logger()?;

    logger.add_trace_logger(trace)?;
    logger.enable_channels("alpha.*.*.*", "alpha")?;

    test_trace_logging_channels()?;
    test_standard_logging_channels()?;

    println!("KTRACE rust alpha demo SDK check passed");
    Ok(())
}
