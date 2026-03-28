use kcli::{ConfigError, HandlerContext, InlineParser};

fn parse_int_or_throw(value: &str) -> Result<i32, String> {
    let parsed = value
        .parse::<i64>()
        .map_err(|_| "expected an integer".to_string())?;
    if parsed < i32::MIN as i64 || parsed > i32::MAX as i64 {
        return Err("integer is out of range".to_string());
    }
    Ok(parsed as i32)
}

fn print_processing_line(context: &HandlerContext, value: &str) {
    if context.value_tokens.is_empty() {
        println!("Processing {}", context.option);
        return;
    }

    if context.value_tokens.len() == 1 {
        println!("Processing {} with value \"{}\"", context.option, value);
        return;
    }

    let joined = context
        .value_tokens
        .iter()
        .map(|token| format!("\"{token}\""))
        .collect::<Vec<_>>()
        .join(",");
    println!("Processing {} with values [{}]", context.option, joined);
}

pub fn get_inline_parser() -> Result<InlineParser, ConfigError> {
    let mut parser = InlineParser::new("--beta")?;
    parser.set_value_handler(
        "-profile",
        |context, value| {
            print_processing_line(context, value);
            Ok(())
        },
        "Select beta runtime profile.",
    )?;
    parser.set_value_handler(
        "-workers",
        |context, value| {
            if !value.is_empty() {
                let _ = parse_int_or_throw(value)?;
            }
            print_processing_line(context, value);
            Ok(())
        },
        "Set beta worker count.",
    )?;
    Ok(parser)
}
