use std::cell::RefCell;
use std::rc::Rc;

use kcli::{InlineParser, Parser};

use crate::support::TestResult;

#[test]
fn optional_value_handler_allows_missing_value() -> TestResult {
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
fn optional_value_handler_accepts_explicit_empty_value() -> TestResult {
    let argv = ["prog", "--build-enable", ""];
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
    assert_eq!(*tokens.borrow(), vec![String::new()]);
    Ok(())
}

#[test]
fn flag_handler_does_not_consume_following_tokens() -> TestResult {
    let argv = ["prog", "--build-meta", "data"];
    let called = Rc::new(RefCell::new(false));
    let positionals = Rc::new(RefCell::new(Vec::<String>::new()));

    let mut parser = Parser::new();
    let mut build = InlineParser::new("build")?;
    {
        let called = Rc::clone(&called);
        build.set_flag_handler(
            "-meta",
            move |_context| {
                *called.borrow_mut() = true;
                Ok(())
            },
            "Record metadata.",
        )?;
    }
    parser.add_inline_parser(build)?;
    {
        let positionals = Rc::clone(&positionals);
        parser.set_positional_handler(move |context| {
            *positionals.borrow_mut() = context.value_tokens.clone();
            Ok(())
        })?;
    }

    parser.parse(&argv)?;
    assert!(*called.borrow());
    assert_eq!(*positionals.borrow(), vec!["data".to_string()]);
    Ok(())
}

#[test]
fn required_value_handler_accepts_dash_prefixed_first_value() -> TestResult {
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
fn required_value_handler_preserves_shell_whitespace() -> TestResult {
    let argv = ["prog", "--name", " Joe "];
    let received_value = Rc::new(RefCell::new(String::new()));
    let received_tokens = Rc::new(RefCell::new(Vec::<String>::new()));

    let mut parser = Parser::new();
    {
        let received_value = Rc::clone(&received_value);
        let received_tokens = Rc::clone(&received_tokens);
        parser.set_value_handler(
            "--name",
            move |context, value| {
                *received_value.borrow_mut() = value.to_string();
                *received_tokens.borrow_mut() = context.value_tokens.clone();
                Ok(())
            },
            "Set the display name.",
        )?;
    }

    parser.parse(&argv)?;
    assert_eq!(received_value.borrow().as_str(), " Joe ");
    assert_eq!(*received_tokens.borrow(), vec![" Joe ".to_string()]);
    Ok(())
}

#[test]
fn required_value_handler_accepts_explicit_empty_value() -> TestResult {
    let argv = ["prog", "--name", ""];
    let received_value = Rc::new(RefCell::new(String::from("sentinel")));
    let received_tokens = Rc::new(RefCell::new(Vec::<String>::new()));

    let mut parser = Parser::new();
    {
        let received_value = Rc::clone(&received_value);
        let received_tokens = Rc::clone(&received_tokens);
        parser.set_value_handler(
            "--name",
            move |context, value| {
                *received_value.borrow_mut() = value.to_string();
                *received_tokens.borrow_mut() = context.value_tokens.clone();
                Ok(())
            },
            "Set the display name.",
        )?;
    }

    parser.parse(&argv)?;
    assert_eq!(received_value.borrow().as_str(), "");
    assert_eq!(*received_tokens.borrow(), vec![String::new()]);
    Ok(())
}

#[test]
fn positional_handler_preserves_explicit_empty_tokens() -> TestResult {
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
