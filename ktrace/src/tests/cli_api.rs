mod common;

use common::run_binary;
use kcli::Parser;
use ktrace::{Logger, OutputOptions, TraceLogger};

fn make_parser(
    logger: &Logger,
    local_trace: &TraceLogger,
) -> Result<Parser, Box<dyn std::error::Error>> {
    let mut parser = Parser::new();
    parser.add_inline_parser(logger.make_inline_parser(local_trace.clone(), "trace")?)?;
    Ok(parser)
}

#[test]
fn trace_examples_include_reference_selector_patterns() -> Result<(), Box<dyn std::error::Error>> {
    let output = run_binary(env!("CARGO_BIN_EXE_core"), "core", &["--trace-examples"]);

    assert!(output.contains("'.abc.xyz'"));
    assert!(output.contains("'*.scheduler.tick'"));
    assert!(output.contains("'{alpha,beta}.*'"));
    assert!(output.contains("'beta.{io,scheduler}.packet'"));
    assert!(output.contains("'{alpha,beta}.net'"));
    Ok(())
}

#[test]
fn inline_parser_resolves_local_namespace_selectors() -> Result<(), Box<dyn std::error::Error>> {
    let logger = Logger::new();
    let local_trace = TraceLogger::new("app")?;
    local_trace.add_channel("startup", ktrace::color("BrightCyan")?)?;
    let other_trace = TraceLogger::new("other")?;
    other_trace.add_channel("startup", ktrace::color("BrightYellow")?)?;

    logger.add_trace_logger(local_trace.clone())?;
    logger.add_trace_logger(other_trace)?;

    let parser = make_parser(&logger, &local_trace)?;
    let argv = vec!["app", "--trace", ".startup"];
    parser.parse(&argv)?;

    assert!(logger.should_trace_channel("app.startup", ""));
    assert!(!logger.should_trace_channel("other.startup", ""));
    Ok(())
}

#[test]
fn inline_parser_output_flags_update_logger_options() -> Result<(), Box<dyn std::error::Error>> {
    let logger = Logger::new();
    let trace = TraceLogger::new("app")?;
    trace.add_channel("startup", ktrace::color("BrightCyan")?)?;
    logger.add_trace_logger(trace.clone())?;

    let parser = make_parser(&logger, &trace)?;
    let argv = vec![
        "app",
        "--trace-files",
        "--trace-functions",
        "--trace-timestamps",
    ];
    parser.parse(&argv)?;

    assert_eq!(
        logger.get_output_options()?,
        OutputOptions {
            filenames: true,
            line_numbers: true,
            function_names: true,
            timestamps: true,
        }
    );
    Ok(())
}
