use std::error::Error;

use ktrace::Logger;

#[path = "support/sdk_beta.rs"]
mod sdk_beta_support;

fn main() -> Result<(), Box<dyn Error>> {
    let logger = Logger::new();
    let trace = sdk_beta_support::get_trace_logger()?;

    logger.add_trace_logger(trace)?;
    logger.enable_channels("beta.*.*.*", "beta")?;
    sdk_beta_support::test_trace_logging_channels()?;

    println!("KTRACE rust beta demo SDK check passed");
    Ok(())
}
