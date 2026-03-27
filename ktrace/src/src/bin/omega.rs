use std::error::Error;

use ktrace::demo::alpha::get_trace_logger as get_alpha_trace_logger;
use ktrace::demo::beta::get_trace_logger as get_beta_trace_logger;
use ktrace::demo::gamma::get_trace_logger as get_gamma_trace_logger;
use ktrace::{ktrace_trace, Logger, TraceLogger};

fn main() -> Result<(), Box<dyn Error>> {
    let argv = std::env::args().collect::<Vec<_>>();

    let logger = Logger::new();
    let app_trace = TraceLogger::new("omega")?;
    app_trace.add_channel("app", ktrace::color("BrightWhite")?)?;
    app_trace.add_channel("startup", ktrace::color("BrightBlue")?)?;

    let alpha_trace = get_alpha_trace_logger()?;
    let beta_trace = get_beta_trace_logger()?;
    let gamma_trace = get_gamma_trace_logger()?;

    logger.add_trace_logger(app_trace.clone())?;
    logger.add_trace_logger(alpha_trace.clone())?;
    logger.add_trace_logger(beta_trace.clone())?;
    logger.add_trace_logger(gamma_trace.clone())?;

    let mut parser = kcli::Parser::new();
    parser.add_inline_parser(logger.make_inline_parser(app_trace.clone(), "trace")?)?;
    parser.parse_or_exit(&argv);

    logger.enable_channels("*.*", app_trace.namespace())?;
    ktrace_trace!(app_trace, "app", "omega app trace")?;
    ktrace_trace!(alpha_trace, "net", "alpha network trace")?;
    ktrace_trace!(beta_trace, "io", "beta io trace")?;
    ktrace_trace!(gamma_trace, "scheduler.tick", "gamma scheduler trace")?;

    println!();
    println!("Usage:");
    println!("  omega --trace '*.*'");
    println!("  omega --trace-namespaces");
    println!();

    Ok(())
}
