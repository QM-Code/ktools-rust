use crate::model::{InlineParserData, ValueArity};

pub(crate) fn build_help_rows(parser: &InlineParserData) -> Vec<(String, String)> {
    let prefix = format!("--{}-", parser.root_name);
    let mut rows = Vec::new();

    if parser.root_value_handler.is_some() && !parser.root_value_description.is_empty() {
        let mut lhs = format!("--{}", parser.root_name);
        if !parser.root_value_placeholder.is_empty() {
            lhs.push(' ');
            lhs.push_str(&parser.root_value_placeholder);
        }
        rows.push((lhs, parser.root_value_description.clone()));
    }

    for (command, binding) in parser.commands.iter() {
        let mut lhs = format!("{prefix}{command}");
        if binding.expects_value {
            match binding.value_arity {
                ValueArity::Optional => lhs.push_str(" [value]"),
                ValueArity::Required => lhs.push_str(" <value>"),
            }
        }
        rows.push((lhs, binding.description.clone()));
    }

    rows
}

pub(crate) fn print_help(root: &str, help_rows: &[(String, String)]) {
    println!();
    println!("Available --{}-* options:", root);

    let max_lhs = help_rows
        .iter()
        .map(|(lhs, _)| lhs.len())
        .max()
        .unwrap_or(0);

    if help_rows.is_empty() {
        println!("  (no options registered)");
    } else {
        for (lhs, rhs) in help_rows {
            let padding = max_lhs.saturating_sub(lhs.len());
            println!("  {lhs}{}{}", " ".repeat(padding + 2), rhs);
        }
    }

    println!();
}
