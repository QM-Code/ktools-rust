use std::cell::RefCell;
use std::rc::Rc;

use kcli::{HandlerContext, InlineParser, Parser};

type TestResult = Result<(), Box<dyn std::error::Error>>;

#[test]
fn parser_empty_parse_succeeds() -> TestResult {
    let argv = ["prog"];
    let parser = Parser::new();
    parser.parse(&argv)?;
    Ok(())
}

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
