use std::error::Error;

use ktrace::demo::alpha::{
    get_trace_logger as get_alpha_trace_logger,
    test_standard_logging_channels as test_alpha_standard_logging_channels,
    test_trace_logging_channels as test_alpha_trace_logging_channels,
};
use ktrace::demo::beta::{
    get_trace_logger as get_beta_trace_logger,
    test_trace_logging_channels as test_beta_trace_logging_channels,
};
use ktrace::demo::gamma::{
    get_trace_logger as get_gamma_trace_logger,
    test_trace_logging_channels as test_gamma_trace_logging_channels,
};
use ktrace::{ktrace_trace, Logger, TraceLogger};

fn main() -> Result<(), Box<dyn Error>> {
    let argv = std::env::args().collect::<Vec<_>>();

    let logger = Logger::new();
    let app_trace = TraceLogger::new("omega")?;
    app_trace.add_channel("app", ktrace::color("BrightCyan")?)?;
    app_trace.add_channel("orchestrator", ktrace::color("BrightYellow")?)?;
    app_trace.add_channel("deep", ktrace::DEFAULT_COLOR)?;
    app_trace.add_channel("deep.branch", ktrace::DEFAULT_COLOR)?;
    app_trace.add_channel("deep.branch.leaf", ktrace::color("LightSalmon1")?)?;

    let alpha_trace = get_alpha_trace_logger()?;
    let beta_trace = get_beta_trace_logger()?;
    let gamma_trace = get_gamma_trace_logger()?;

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
    test_alpha_trace_logging_channels()?;
    test_beta_trace_logging_channels()?;
    test_gamma_trace_logging_channels()?;
    test_alpha_standard_logging_channels()?;
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
