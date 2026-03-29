use std::error::Error;

use ktrace::{ktrace_info, ktrace_trace, Logger, TraceLogger};

#[path = "support/sdk_alpha.rs"]
mod sdk_alpha_support;

fn executable_name(path: Option<&str>) -> &str {
    match path {
        Some(path) if !path.is_empty() => path,
        _ => "app",
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let argv = std::env::args().collect::<Vec<_>>();
    let exe_name = executable_name(argv.first().map(String::as_str));

    let logger = Logger::new();
    let app_trace = TraceLogger::new("core")?;
    app_trace.add_channel("app", ktrace::color("BrightCyan")?)?;
    app_trace.add_channel("startup", ktrace::color("BrightYellow")?)?;
    let alpha_trace = sdk_alpha_support::get_trace_logger()?;

    logger.add_trace_logger(app_trace.clone())?;
    logger.add_trace_logger(alpha_trace)?;

    let mut parser = kcli::Parser::new();
    parser.add_inline_parser(logger.make_inline_parser(app_trace.clone(), "trace")?)?;

    logger.enable_channel(".app", app_trace.namespace())?;
    ktrace_trace!(app_trace, "app", "core initialized local trace channels")?;
    parser.parse_or_exit(&argv);

    ktrace_trace!(
        app_trace,
        "app",
        "cli processing enabled, use --trace for options"
    )?;
    ktrace_trace!(
        app_trace,
        "startup",
        "testing imported tracing, use --trace '*.*' to view imported channels"
    )?;
    sdk_alpha_support::test_trace_logging_channels()?;
    ktrace_info!(
        app_trace,
        "KTRACE rust demo core import/integration check passed"
    )?;

    println!();
    println!("Usage:");
    println!("  {exe_name} --trace");
    println!("  {exe_name} --trace '.app'");
    println!();

    Ok(())
}
