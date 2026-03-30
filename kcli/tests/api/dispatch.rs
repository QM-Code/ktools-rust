use std::cell::RefCell;
use std::rc::Rc;

use kcli::{HandlerContext, InlineParser, Parser};

use crate::support::TestResult;

#[test]
fn parser_empty_parse_succeeds() -> TestResult {
    let argv = ["prog"];
    let parser = Parser::new();
    parser.parse(&argv)?;
    Ok(())
}

#[test]
fn end_user_known_options_with_unknown_option_error() -> TestResult {
    let argv = [
        "prog",
        "--verbose",
        "pos1",
        "--output",
        "stdout",
        "--bogus",
        "pos2",
    ];
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
fn parser_can_be_reused_across_parses() -> TestResult {
    let argv = ["prog", "-v"];
    let calls = Rc::new(RefCell::new(0));

    let mut parser = Parser::new();
    parser.add_alias("-v", "--verbose", &[] as &[&str])?;

    {
        let calls = Rc::clone(&calls);
        parser.set_flag_handler(
            "--verbose",
            move |_context| {
                *calls.borrow_mut() += 1;
                Ok(())
            },
            "Enable verbose logging.",
        )?;
    }

    parser.parse(&argv)?;
    parser.parse(&argv)?;
    assert_eq!(*calls.borrow(), 2);
    Ok(())
}

#[test]
fn single_pass_processing_consumes_inline_end_user_and_positionals() -> TestResult {
    let argv = [
        "prog",
        "tail",
        "--alpha-message",
        "hello",
        "--output",
        "stdout",
    ];
    let alpha_message = Rc::new(RefCell::new(String::new()));
    let output = Rc::new(RefCell::new(String::new()));
    let positionals = Rc::new(RefCell::new(Vec::<String>::new()));

    let mut parser = Parser::new();
    let mut alpha = InlineParser::new("alpha")?;
    {
        let alpha_message = Rc::clone(&alpha_message);
        alpha.set_value_handler(
            "-message",
            move |_context, value| {
                *alpha_message.borrow_mut() = value.to_string();
                Ok(())
            },
            "Set alpha message.",
        )?;
    }
    parser.add_inline_parser(alpha)?;
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
    {
        let positionals = Rc::clone(&positionals);
        parser.set_positional_handler(move |context| {
            *positionals.borrow_mut() = context.value_tokens.clone();
            Ok(())
        })?;
    }

    parser.parse(&argv)?;
    assert_eq!(alpha_message.borrow().as_str(), "hello");
    assert_eq!(output.borrow().as_str(), "stdout");
    assert_eq!(*positionals.borrow(), vec!["tail".to_string()]);
    Ok(())
}
