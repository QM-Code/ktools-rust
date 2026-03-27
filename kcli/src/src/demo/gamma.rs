use crate::demo::common::print_processing_line;
use crate::{ConfigError, InlineParser};

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

