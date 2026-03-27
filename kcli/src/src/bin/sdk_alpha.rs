use std::error::Error;

use kcli::demo::alpha::get_inline_parser;
use kcli::Parser;

fn main() -> Result<(), Box<dyn Error>> {
    let argv = std::env::args().collect::<Vec<_>>();

    let mut parser = Parser::new();
    parser.add_inline_parser(get_inline_parser()?)?;
    parser.parse_or_exit(&argv);

    println!("KCLI rust alpha demo SDK check passed");
    Ok(())
}
