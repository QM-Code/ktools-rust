use std::error::Error;

use ktrace::Logger;

#[path = "support/sdk_alpha.rs"]
mod sdk_alpha_support;

fn main() -> Result<(), Box<dyn Error>> {
    let logger = Logger::new();
    let trace = sdk_alpha_support::get_trace_logger()?;

    logger.add_trace_logger(trace)?;
    logger.enable_channels("alpha.*.*.*", "alpha")?;

    sdk_alpha_support::test_trace_logging_channels()?;
    sdk_alpha_support::test_standard_logging_channels()?;

    println!("KTRACE rust alpha demo SDK check passed");
    Ok(())
}
