use std::error::Error;

use ktrace::Logger;

#[path = "support/sdk_gamma.rs"]
mod sdk_gamma_support;

fn main() -> Result<(), Box<dyn Error>> {
    let logger = Logger::new();
    let trace = sdk_gamma_support::get_trace_logger()?;

    logger.add_trace_logger(trace)?;
    logger.enable_channels("gamma.*.*.*", "gamma")?;
    sdk_gamma_support::test_trace_logging_channels()?;

    println!("KTRACE rust gamma demo SDK check passed");
    Ok(())
}
