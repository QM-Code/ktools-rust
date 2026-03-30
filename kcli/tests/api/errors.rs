use kcli::{InlineParser, Parser};

use crate::support::TestResult;

#[test]
fn end_user_handler_normalization_rejects_single_dash() -> TestResult {
    let mut parser = Parser::new();
    let error = parser
        .set_flag_handler("-verbose", |_context| Ok(()), "Enable verbose logging.")
        .unwrap_err();
    assert!(error.to_string().contains("must use '--name' or 'name'"));
    Ok(())
}

#[test]
fn inline_missing_root_value_handler_errors() -> TestResult {
    let argv = ["prog", "--build", "fast"];
    let mut parser = Parser::new();
    parser.add_inline_parser(InlineParser::new("--build")?)?;

    let error = parser.parse(&argv).unwrap_err();
    assert_eq!(error.option(), "--build");
    assert!(error
        .to_string()
        .contains("unknown value for option '--build'"));
    Ok(())
}

#[test]
fn required_value_handler_rejects_missing_value() -> TestResult {
    let argv = ["prog", "--build-value"];

    let mut parser = Parser::new();
    let mut build = InlineParser::new("build")?;
    build.set_value_handler("-value", |_context, _value| Ok(()), "Set build value.")?;
    parser.add_inline_parser(build)?;

    let error = parser.parse(&argv).unwrap_err();
    assert_eq!(error.option(), "--build-value");
    assert!(error.to_string().contains("requires a value"));
    Ok(())
}

#[test]
fn unknown_inline_option_errors() -> TestResult {
    let argv = ["prog", "--build-unknown"];
    let mut parser = Parser::new();
    parser.add_inline_parser(InlineParser::new("--build")?)?;

    let error = parser.parse(&argv).unwrap_err();
    assert_eq!(error.option(), "--build-unknown");
    assert!(error.to_string().contains("unknown option --build-unknown"));
    Ok(())
}

#[test]
fn unknown_option_reports_double_dash() {
    let argv = ["prog", "--"];
    let parser = Parser::new();
    let error = parser.parse(&argv).unwrap_err();
    assert_eq!(error.option(), "--");
}

#[test]
fn unknown_option_throws_cli_error() {
    let argv = ["prog", "--bogus"];
    let parser = Parser::new();
    let error = parser.parse(&argv).unwrap_err();
    assert_eq!(error.option(), "--bogus");
    assert!(error.to_string().contains("unknown option --bogus"));
}

#[test]
fn option_handler_error_returns_cli_error() -> TestResult {
    let argv = ["prog", "--verbose"];
    let mut parser = Parser::new();
    parser.set_flag_handler(
        "--verbose",
        |_context| Err("option boom".to_string()),
        "Enable verbose logging.",
    )?;

    let error = parser.parse(&argv).unwrap_err();
    assert_eq!(error.option(), "--verbose");
    assert!(error.to_string().contains("option boom"));
    assert!(error.to_string().contains("--verbose"));
    Ok(())
}

#[test]
fn positional_handler_error_returns_cli_error() -> TestResult {
    let argv = ["prog", "tail"];
    let mut parser = Parser::new();
    parser.set_positional_handler(|_context| Err("positional boom".to_string()))?;

    let error = parser.parse(&argv).unwrap_err();
    assert_eq!(error.option(), "");
    assert!(error.to_string().contains("positional boom"));
    Ok(())
}
