use std::cell::RefCell;
use std::rc::Rc;

use kcli::{CliError, HandlerContext, InlineParser, Parser};

#[test]
fn parser_empty_parse_succeeds() -> Result<(), Box<dyn std::error::Error>> {
    let argv = ["prog"];
    let parser = Parser::new();
    parser.parse(&argv)?;
    Ok(())
}

#[test]
fn end_user_known_options_with_unknown_option_error(
) -> Result<(), Box<dyn std::error::Error>> {
    let argv = ["prog", "--verbose", "pos1", "--output", "stdout", "--bogus", "pos2"];
    let verbose = Rc::new(RefCell::new(false));
    let output = Rc::new(RefCell::new(String::new()));
    let positionals = Rc::new(RefCell::new(Vec::<String>::new()));

    let mut parser = Parser::new();

    {
        let verbose = Rc::clone(&verbose);
        parser.set_flag_handler(
            "verbose",
            move |_context| {
                *verbose.borrow_mut() = true;
                Ok(())
            },
            "Enable verbose logging.",
        )?;
    }

    {
        let output = Rc::clone(&output);
        parser.set_value_handler(
            "output",
            move |_context, value| {
                *output.borrow_mut() = value.to_string();
                Ok(())
            },
            "Set output target.",
        )?;
    }

    {
        let positionals = Rc::clone(&positionals);
        parser.set_positional_handler(move |context: &HandlerContext| {
            positionals
                .borrow_mut()
                .extend(context.value_tokens.iter().cloned());
            Ok(())
        })?;
    }

    let error = parser.parse(&argv).unwrap_err();
    assert_eq!(error.option(), "--bogus");
    assert!(error.to_string().contains("unknown option --bogus"));
    assert!(!*verbose.borrow());
    assert!(output.borrow().is_empty());
    assert!(positionals.borrow().is_empty());
    Ok(())
}

#[test]
fn add_alias_rewrites_tokens() -> Result<(), Box<dyn std::error::Error>> {
    let argv = ["prog", "-v", "tail"];
    let seen_option = Rc::new(RefCell::new(String::new()));

    let mut parser = Parser::new();
    parser.add_alias("-v", "--verbose", &[] as &[&str])?;

    {
        let seen_option = Rc::clone(&seen_option);
        parser.set_flag_handler(
            "--verbose",
            move |context| {
                *seen_option.borrow_mut() = context.option.clone();
                Ok(())
            },
            "Enable verbose logging.",
        )?;
    }

    parser.parse(&argv)?;
    assert_eq!(seen_option.borrow().as_str(), "--verbose");
    Ok(())
}

#[test]
fn add_alias_preset_tokens_apply_to_inline_root_values(
) -> Result<(), Box<dyn std::error::Error>> {
    let argv = ["prog", "-c"];
    let seen = Rc::new(RefCell::new(String::new()));
    let tokens = Rc::new(RefCell::new(Vec::<String>::new()));

    let mut parser = Parser::new();
    let mut config = InlineParser::new("--config")?;
    {
        let seen = Rc::clone(&seen);
        let tokens = Rc::clone(&tokens);
        config.set_root_value_handler_with_help(
            move |context, value| {
                *seen.borrow_mut() = value.to_string();
                *tokens.borrow_mut() = context.value_tokens.clone();
                Ok(())
            },
            "<assignment>",
            "Store a config assignment.",
        )?;
    }
    parser.add_inline_parser(config)?;
    parser.add_alias("-c", "--config", &["user-file=/tmp/user.json"])?;

    parser.parse(&argv)?;
    assert_eq!(seen.borrow().as_str(), "user-file=/tmp/user.json");
    assert_eq!(*tokens.borrow(), vec!["user-file=/tmp/user.json".to_string()]);
    Ok(())
}

#[test]
fn optional_value_handler_allows_missing_value(
) -> Result<(), Box<dyn std::error::Error>> {
    let argv = ["prog", "--build-enable"];
    let value = Rc::new(RefCell::new(String::from("sentinel")));
    let tokens = Rc::new(RefCell::new(Vec::<String>::new()));

    let mut parser = Parser::new();
    let mut build = InlineParser::new("build")?;
    {
        let value = Rc::clone(&value);
        let tokens = Rc::clone(&tokens);
        build.set_optional_value_handler(
            "-enable",
            move |context, raw_value| {
                *value.borrow_mut() = raw_value.to_string();
                *tokens.borrow_mut() = context.value_tokens.clone();
                Ok(())
            },
            "Enable build mode.",
        )?;
    }
    parser.add_inline_parser(build)?;

    parser.parse(&argv)?;
    assert_eq!(value.borrow().as_str(), "");
    assert!(tokens.borrow().is_empty());
    Ok(())
}

#[test]
fn required_value_handler_accepts_dash_prefixed_first_value(
) -> Result<(), Box<dyn std::error::Error>> {
    let argv = ["prog", "--build-value", "-debug"];
    let value = Rc::new(RefCell::new(String::new()));

    let mut parser = Parser::new();
    let mut build = InlineParser::new("build")?;
    {
        let value = Rc::clone(&value);
        build.set_value_handler(
            "-value",
            move |_context, raw_value| {
                *value.borrow_mut() = raw_value.to_string();
                Ok(())
            },
            "Set build value.",
        )?;
    }
    parser.add_inline_parser(build)?;

    parser.parse(&argv)?;
    assert_eq!(value.borrow().as_str(), "-debug");
    Ok(())
}

#[test]
fn positional_handler_preserves_explicit_empty_tokens(
) -> Result<(), Box<dyn std::error::Error>> {
    let argv = ["prog", "", "tail"];
    let positionals = Rc::new(RefCell::new(Vec::<String>::new()));

    let mut parser = Parser::new();
    {
        let positionals = Rc::clone(&positionals);
        parser.set_positional_handler(move |context| {
            *positionals.borrow_mut() = context.value_tokens.clone();
            Ok(())
        })?;
    }

    parser.parse(&argv)?;
    assert_eq!(
        *positionals.borrow(),
        vec![String::new(), "tail".to_string()]
    );
    Ok(())
}

#[test]
fn unknown_option_throws_cli_error() -> Result<(), Box<dyn std::error::Error>> {
    let argv = ["prog", "--bogus"];
    let parser = Parser::new();
    let error = parser.parse(&argv).unwrap_err();
    assert_eq!(error.option(), "--bogus");
    assert!(error.to_string().contains("unknown option --bogus"));
    Ok(())
}

#[test]
fn option_handler_error_returns_cli_error() -> Result<(), Box<dyn std::error::Error>> {
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
fn add_alias_rejects_preset_values_for_flag_targets(
) -> Result<(), Box<dyn std::error::Error>> {
    let argv = ["prog", "-v"];
    let mut parser = Parser::new();
    parser.add_alias("-v", "--verbose", &["unexpected"])?;
    parser.set_flag_handler("--verbose", |_context| Ok(()), "Enable verbose logging.")?;

    let error = parser.parse(&argv).unwrap_err();
    assert_eq!(error.option(), "-v");
    assert!(error.to_string().contains("does not accept values"));
    Ok(())
}

#[test]
fn duplicate_inline_root_rejected() -> Result<(), Box<dyn std::error::Error>> {
    let mut parser = Parser::new();
    parser.add_inline_parser(InlineParser::new("--build")?)?;
    let error = parser
        .add_inline_parser(InlineParser::new("build")?)
        .unwrap_err();
    assert!(error
        .to_string()
        .contains("kcli inline parser root '--build' is already registered"));
    Ok(())
}

#[test]
fn cli_error_default_message_is_used() {
    let error = CliError::new("", "");
    assert_eq!(error.to_string(), "kcli parse failed");
}

