use std::error::Error;

use kcli::demo::alpha::get_inline_parser;
use kcli::Parser;

fn executable_name(path: Option<&str>) -> &str {
    match path {
        Some(path) if !path.is_empty() => path,
        _ => "app",
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let argv = std::env::args().collect::<Vec<_>>();
    let exe_name = executable_name(argv.first().map(String::as_str));

    let mut parser = Parser::new();
    parser.add_inline_parser(get_inline_parser()?)?;

    parser.add_alias("-v", "--verbose", &[] as &[&str])?;
    parser.add_alias("-out", "--output", &[] as &[&str])?;
    parser.add_alias("-a", "--alpha-enable", &[] as &[&str])?;

    parser.set_flag_handler("--verbose", |_context| Ok(()), "Enable verbose app logging.")?;
    parser.set_value_handler(
        "--output",
        |_context, _value| Ok(()),
        "Set app output target.",
    )?;

    parser.parse_or_exit(&argv);

    println!();
    println!("KCLI rust demo core import/integration check passed");
    println!();
    println!("Usage:");
    println!("  {exe_name} --alpha");
    println!("  {exe_name} --output stdout");
    println!();
    println!("Enabled inline roots:");
    println!("  --alpha");
    println!();

    Ok(())
}

