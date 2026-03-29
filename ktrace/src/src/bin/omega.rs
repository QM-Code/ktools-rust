use std::error::Error;

use ktrace::{ktrace_trace, Logger, TraceLogger};

#[path = "support/sdk_alpha.rs"]
mod sdk_alpha_support;
#[path = "support/sdk_beta.rs"]
mod sdk_beta_support;
#[path = "support/sdk_gamma.rs"]
mod sdk_gamma_support;

fn main() -> Result<(), Box<dyn Error>> {
    let argv = std::env::args().collect::<Vec<_>>();

    let logger = Logger::new();
    let app_trace = TraceLogger::new("omega")?;
    app_trace.add_channel("app", ktrace::color("BrightCyan")?)?;
    app_trace.add_channel("orchestrator", ktrace::color("BrightYellow")?)?;
    app_trace.add_channel("deep", ktrace::DEFAULT_COLOR)?;
    app_trace.add_channel("deep.branch", ktrace::DEFAULT_COLOR)?;
    app_trace.add_channel("deep.branch.leaf", ktrace::color("LightSalmon1")?)?;

    let alpha_trace = sdk_alpha_support::get_trace_logger()?;
    let beta_trace = sdk_beta_support::get_trace_logger()?;
    let gamma_trace = sdk_gamma_support::get_trace_logger()?;

    logger.add_trace_logger(app_trace.clone())?;
    logger.add_trace_logger(alpha_trace.clone())?;
    logger.add_trace_logger(beta_trace.clone())?;
    logger.add_trace_logger(gamma_trace.clone())?;

    let mut parser = kcli::Parser::new();
    parser.add_inline_parser(logger.make_inline_parser(app_trace.clone(), "trace")?)?;

    logger.enable_channel(".app", app_trace.namespace())?;
    ktrace_trace!(app_trace, "app", "omega initialized local trace channels")?;
    logger.disable_channel(".app", app_trace.namespace())?;
    parser.parse_or_exit(&argv);

    ktrace_trace!(
        app_trace,
        "app",
        "cli processing enabled, use --trace for options"
    )?;
    ktrace_trace!(
        app_trace,
        "app",
        "testing external tracing, use --trace '*.*' to view top-level channels"
    )?;
    ktrace_trace!(
        app_trace,
        "deep.branch.leaf",
        "omega trace test on channel 'deep.branch.leaf'"
    )?;
    sdk_alpha_support::test_trace_logging_channels()?;
    sdk_beta_support::test_trace_logging_channels()?;
    sdk_gamma_support::test_trace_logging_channels()?;
    sdk_alpha_support::test_standard_logging_channels()?;
    app_trace.trace("orchestrator", "omega completed imported SDK trace checks")?;
    app_trace.info("testing...")?;
    app_trace.warn("testing...")?;
    app_trace.error("testing...")?;

    println!();
    println!("Usage:");
    println!("  omega --trace '*.*'");
    println!("  omega --trace-namespaces");
    println!();

    Ok(())
}
