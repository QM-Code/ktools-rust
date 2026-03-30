use std::cell::RefCell;
use std::rc::Rc;

use kcli::{InlineParser, Parser};

use crate::support::TestResult;

#[test]
fn inline_parser_rejects_invalid_root() {
    let error = match InlineParser::new("-build") {
        Ok(_) => panic!("InlineParser::new should reject single-dash roots"),
        Err(error) => error,
    };
    assert!(error
        .to_string()
        .contains("kcli root must use '--root' or 'root'"));
}

#[test]
fn inline_handler_normalization_accepts_short_and_full_forms() -> TestResult {
    let argv = ["prog", "--build-flag", "--build-value", "data"];
    let flag = Rc::new(RefCell::new(false));
    let value = Rc::new(RefCell::new(String::new()));

    let mut parser = Parser::new();
    let mut build = InlineParser::new("build")?;
    {
        let flag = Rc::clone(&flag);
        build.set_flag_handler(
            "-flag",
            move |_context| {
                *flag.borrow_mut() = true;
                Ok(())
            },
            "Enable build flag.",
        )?;
    }
    {
        let value = Rc::clone(&value);
        build.set_value_handler(
            "--build-value",
            move |_context, raw_value| {
                *value.borrow_mut() = raw_value.to_string();
                Ok(())
            },
            "Set build value.",
        )?;
    }
    parser.add_inline_parser(build)?;

    parser.parse(&argv)?;
    assert!(*flag.borrow());
    assert_eq!(value.borrow().as_str(), "data");
    Ok(())
}

#[test]
fn inline_handler_normalization_rejects_wrong_root() -> TestResult {
    let mut inline_parser = InlineParser::new("--build")?;
    let error = inline_parser
        .set_flag_handler("--other-flag", |_context| Ok(()), "Enable other flag.")
        .unwrap_err();
    assert!(error
        .to_string()
        .contains("kcli inline handler option must use '-name' or '--build-name'"));
    Ok(())
}

#[test]
fn inline_root_value_handler_joins_tokens() -> TestResult {
    let argv = ["prog", "--build", "fast", "mode"];
    let received_value = Rc::new(RefCell::new(String::new()));
    let received_tokens = Rc::new(RefCell::new(Vec::<String>::new()));
    let received_option = Rc::new(RefCell::new(String::new()));

    let mut parser = Parser::new();
    let mut build = InlineParser::new("build")?;
    {
        let received_value = Rc::clone(&received_value);
        let received_tokens = Rc::clone(&received_tokens);
        let received_option = Rc::clone(&received_option);
        build.set_root_value_handler(move |context, value| {
            *received_value.borrow_mut() = value.to_string();
            *received_tokens.borrow_mut() = context.value_tokens.clone();
            *received_option.borrow_mut() = context.option.clone();
            Ok(())
        })?;
    }
    parser.add_inline_parser(build)?;

    parser.parse(&argv)?;
    assert_eq!(received_value.borrow().as_str(), "fast mode");
    assert_eq!(
        *received_tokens.borrow(),
        vec!["fast".to_string(), "mode".to_string()]
    );
    assert_eq!(received_option.borrow().as_str(), "--build");
    Ok(())
}

#[test]
fn inline_parser_root_override_applies() -> TestResult {
    let argv = ["prog", "--newgamma-tag", "prod"];
    let tag = Rc::new(RefCell::new(String::new()));

    let mut parser = Parser::new();
    let mut gamma = InlineParser::new("--gamma")?;
    {
        let tag = Rc::clone(&tag);
        gamma.set_value_handler(
            "-tag",
            move |_context, value| {
                *tag.borrow_mut() = value.to_string();
                Ok(())
            },
            "Set gamma tag.",
        )?;
    }
    gamma.set_root("--newgamma")?;
    parser.add_inline_parser(gamma)?;

    parser.parse(&argv)?;
    assert_eq!(tag.borrow().as_str(), "prod");
    Ok(())
}

#[test]
fn duplicate_inline_root_rejected() -> TestResult {
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
