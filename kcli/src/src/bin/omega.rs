use std::error::Error;

use kcli::demo::alpha::get_inline_parser as get_alpha_inline_parser;
use kcli::demo::beta::get_inline_parser as get_beta_inline_parser;
use kcli::demo::gamma::get_inline_parser as get_gamma_inline_parser;
use kcli::{InlineParser, Parser};

fn main() -> Result<(), Box<dyn Error>> {
    let argv = std::env::args().collect::<Vec<_>>();

    let mut parser = Parser::new();
    let alpha_parser = get_alpha_inline_parser()?;
    let beta_parser = get_beta_inline_parser()?;
    let mut gamma_parser = get_gamma_inline_parser()?;
    gamma_parser.set_root("--newgamma")?;

    let mut build_parser = InlineParser::new("--build")?;
    build_parser.set_value_handler(
        "-profile",
        |_context, _value| Ok(()),
        "Set build profile.",
    )?;
    build_parser.set_flag_handler("-clean", |_context| Ok(()), "Enable clean build.")?;

    parser.add_inline_parser(alpha_parser)?;
    parser.add_inline_parser(beta_parser)?;
    parser.add_inline_parser(gamma_parser)?;
    parser.add_inline_parser(build_parser)?;

    parser.add_alias("-v", "--verbose", &[] as &[&str])?;
    parser.add_alias("-out", "--output", &[] as &[&str])?;
    parser.add_alias("-a", "--alpha-enable", &[] as &[&str])?;
    parser.add_alias("-b", "--build-profile", &[] as &[&str])?;

    parser.set_flag_handler("--verbose", |_context| Ok(()), "Enable verbose app logging.")?;
    parser.set_value_handler(
        "--output",
        |_context, _value| Ok(()),
        "Set app output target.",
    )?;
    parser.set_positional_handler(|_context| Ok(()))?;

    parser.parse_or_exit(&argv);

    println!();
    println!("Usage:");
    println!("  kcli_demo_omega --<root>");
    println!();
    println!("Enabled --<root> prefixes:");
    println!("  --alpha");
    println!("  --beta");
    println!("  --newgamma (gamma override)");
    println!();

    Ok(())
}

