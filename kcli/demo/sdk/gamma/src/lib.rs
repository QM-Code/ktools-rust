use kcli::{ConfigError, HandlerContext, InlineParser};

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
    let mut parser = InlineParser::new("--gamma")?;
    parser.set_optional_value_handler(
        "-strict",
        |context, value| {
            print_processing_line(context, value);
            Ok(())
        },
        "Enable strict gamma mode.",
    )?;
    parser.set_value_handler(
        "-tag",
        |context, value| {
            print_processing_line(context, value);
            Ok(())
        },
        "Set a gamma tag label.",
    )?;
    Ok(parser)
}
