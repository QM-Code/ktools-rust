use crate::demo::common::print_processing_line;
use crate::{ConfigError, InlineParser};

pub fn get_inline_parser() -> Result<InlineParser, ConfigError> {
    let mut parser = InlineParser::new("--alpha")?;
    parser.set_value_handler(
        "-message",
        |context, value| {
            print_processing_line(context, value);
            Ok(())
        },
        "Set alpha message label.",
    )?;
    parser.set_optional_value_handler(
        "-enable",
        |context, value| {
            print_processing_line(context, value);
            Ok(())
        },
        "Enable alpha processing.",
    )?;
    Ok(parser)
}

