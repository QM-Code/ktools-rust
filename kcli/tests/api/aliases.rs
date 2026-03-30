use std::cell::RefCell;
use std::rc::Rc;

use kcli::{InlineParser, Parser};

use crate::support::TestResult;

#[test]
fn add_alias_rewrites_tokens() -> TestResult {
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
fn add_alias_preset_tokens_append_to_value_handlers() -> TestResult {
    let argv = ["prog", "-c", "settings.json"];
    let option = Rc::new(RefCell::new(String::new()));
    let value = Rc::new(RefCell::new(String::new()));
    let value_tokens = Rc::new(RefCell::new(Vec::<String>::new()));

    let mut parser = Parser::new();
    parser.add_alias("-c", "--config-load", &["user-file"])?;

    {
        let option = Rc::clone(&option);
        let value = Rc::clone(&value);
        let value_tokens = Rc::clone(&value_tokens);
        parser.set_value_handler(
            "--config-load",
            move |context, captured| {
                *option.borrow_mut() = context.option.clone();
                *value.borrow_mut() = captured.to_string();
                *value_tokens.borrow_mut() = context.value_tokens.clone();
                Ok(())
            },
            "Load config.",
        )?;
    }

    parser.parse(&argv)?;
    assert_eq!(option.borrow().as_str(), "--config-load");
    assert_eq!(value.borrow().as_str(), "user-file settings.json");
    assert_eq!(
        *value_tokens.borrow(),
        vec!["user-file".to_string(), "settings.json".to_string()]
    );
    Ok(())
}

#[test]
fn add_alias_preset_tokens_satisfy_required_values() -> TestResult {
    let argv = ["prog", "-p"];
    let value = Rc::new(RefCell::new(String::new()));
    let value_tokens = Rc::new(RefCell::new(Vec::<String>::new()));

    let mut parser = Parser::new();
    parser.add_alias("-p", "--profile", &["release"])?;

    {
        let value = Rc::clone(&value);
        let value_tokens = Rc::clone(&value_tokens);
        parser.set_value_handler(
            "--profile",
            move |context, captured| {
                assert_eq!(context.option, "--profile");
                *value.borrow_mut() = captured.to_string();
                *value_tokens.borrow_mut() = context.value_tokens.clone();
                Ok(())
            },
            "Set the active profile.",
        )?;
    }

    parser.parse(&argv)?;
    assert_eq!(value.borrow().as_str(), "release");
    assert_eq!(*value_tokens.borrow(), vec!["release".to_string()]);
    Ok(())
}

#[test]
fn add_alias_preset_tokens_apply_to_inline_root_values() -> TestResult {
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
    assert_eq!(
        *tokens.borrow(),
        vec!["user-file=/tmp/user.json".to_string()]
    );
    Ok(())
}

#[test]
fn add_alias_rewrites_after_double_dash() -> TestResult {
    let argv = ["prog", "--", "-v"];
    let verbose = Rc::new(RefCell::new(false));

    let mut parser = Parser::new();
    parser.add_alias("-v", "--verbose", &[] as &[&str])?;
    {
        let verbose = Rc::clone(&verbose);
        parser.set_flag_handler(
            "--verbose",
            move |_context| {
                *verbose.borrow_mut() = true;
                Ok(())
            },
            "Enable verbose logging.",
        )?;
    }

    let error = parser.parse(&argv).unwrap_err();
    assert_eq!(error.option(), "--");
    assert!(!*verbose.borrow());
    Ok(())
}

#[test]
fn alias_does_not_rewrite_required_value_tokens() -> TestResult {
    let argv = ["prog", "--output", "-v"];
    let verbose = Rc::new(RefCell::new(false));
    let output = Rc::new(RefCell::new(String::new()));

    let mut parser = Parser::new();
    parser.add_alias("-v", "--verbose", &[] as &[&str])?;

    {
        let verbose = Rc::clone(&verbose);
        parser.set_flag_handler(
            "--verbose",
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
            "--output",
            move |_context, value| {
                *output.borrow_mut() = value.to_string();
                Ok(())
            },
            "Set output target.",
        )?;
    }

    parser.parse(&argv)?;
    assert!(!*verbose.borrow());
    assert_eq!(output.borrow().as_str(), "-v");
    Ok(())
}

#[test]
fn add_alias_rejects_invalid_alias() -> TestResult {
    let mut parser = Parser::new();
    let error = parser
        .add_alias("--verbose", "--output", &[] as &[&str])
        .unwrap_err();
    assert!(error.to_string().contains("single-dash form"));
    Ok(())
}

#[test]
fn add_alias_rejects_invalid_target() -> TestResult {
    let mut parser = Parser::new();
    let error = parser
        .add_alias("-v", "--bad target", &[] as &[&str])
        .unwrap_err();
    assert!(error.to_string().contains("double-dash form"));
    Ok(())
}

#[test]
fn add_alias_rejects_single_dash_target() -> TestResult {
    let mut parser = Parser::new();
    let error = parser.add_alias("-a", "-b", &[] as &[&str]).unwrap_err();
    assert!(error.to_string().contains("double-dash form"));
    Ok(())
}

#[test]
fn add_alias_rejects_preset_values_for_flag_targets() -> TestResult {
    let argv = ["prog", "-v"];
    let mut parser = Parser::new();
    parser.add_alias("-v", "--verbose", &["unexpected"])?;
    parser.set_flag_handler("--verbose", |_context| Ok(()), "Enable verbose logging.")?;

    let error = parser.parse(&argv).unwrap_err();
    assert_eq!(error.option(), "-v");
    assert!(error.to_string().contains("does not accept values"));
    Ok(())
}
